use std::collections::BTreeSet;

use crate::activity::activity_iso_from_ms;
use crate::common::{row_string, ParamsBuilder};
use crate::database::DatabaseService;
use crate::game_log::ensure_game_log_tables;
use crate::ownership::{owner_id_for_filter, OwnerId};
use crate::Error;

use super::aggregate::{
    access_split, series, series_bucket_for_range, summarize, summarize_previous, worlds,
};
use super::cache::{
    read_cached_page, source_cursor, write_cached_page, CachedPage, PAYLOAD_VERSION,
};
use super::lock::with_activity_page_build_lock;
use super::people::people;
use super::spans::{read_location_spans, LocationSpan};
use super::types::{ActivityPageBuildInput, ActivityPageCoverage, ActivityPageView};

const DAY_MS: i64 = 86_400_000;
const MINUTE_MS: i64 = 60_000;
const TOP_WORLD_LIMIT: usize = 10;

pub fn activity_page_view_build(
    db: &DatabaseService,
    input: ActivityPageBuildInput,
) -> Result<ActivityPageView, Error> {
    if input.owner_user_id.as_str().trim().is_empty() || input.range_days < 0 {
        return Ok(empty_activity_page_view(&input));
    }
    let range_days = input.range_days;
    with_activity_page_build_lock(db, input.owner_user_id.as_str(), || {
        let cursor = source_cursor(db, &input.owner_user_id)?;
        let cached = read_cached_page(db, &input.owner_user_id, range_days)?;
        let window = window_bounds(&input, range_days);

        if !input.force_refresh {
            if let Some(cached) = &cached {
                if is_reusable(cached, &cursor, &input, &window) {
                    return Ok(cached.view.clone());
                }
            }
        }

        match build_fresh(db, &input, range_days, &cursor, &window) {
            Ok(view) => {
                if !view.has_open_tail {
                    write_cached_page(db, &input.owner_user_id, range_days, &view)?;
                }
                Ok(view)
            }
            Err(error) => match cached {
                Some(cached) => {
                    tracing::warn!(
                        range_days,
                        error = %error,
                        "activity page rebuild failed; serving stale cache"
                    );
                    Ok(ActivityPageView {
                        stale: true,
                        ..cached.view
                    })
                }
                None => Err(error),
            },
        }
    })
}

fn is_reusable(
    cached: &CachedPage,
    cursor: &str,
    input: &ActivityPageBuildInput,
    window: &WindowBounds,
) -> bool {
    cached.payload_version == PAYLOAD_VERSION
        && cached.built_from_cursor == cursor
        && cached.view.utc_offset_minutes == input.utc_offset_minutes
        && cached.view.window_from_ms == window.from_ms.unwrap_or(0)
        && cached.view.window_to_ms == window.to_ms
        && !cached.view.has_open_tail
}

fn empty_activity_page_view(input: &ActivityPageBuildInput) -> ActivityPageView {
    ActivityPageView {
        range_days: input.range_days.max(0),
        utc_offset_minutes: input.utc_offset_minutes,
        built_at: activity_iso_from_ms(input.now_ms),
        ..Default::default()
    }
}

pub(super) struct WindowBounds {
    pub(super) from_ms: Option<i64>,
    pub(super) to_ms: i64,
}

fn window_bounds(input: &ActivityPageBuildInput, range_days: i64) -> WindowBounds {
    let offset_ms = input.utc_offset_minutes * MINUTE_MS;
    let local_now_ms = input.now_ms + offset_ms;
    let local_day_end_ms = local_now_ms.div_euclid(DAY_MS) * DAY_MS + DAY_MS;
    let to_ms = local_day_end_ms - offset_ms;
    WindowBounds {
        from_ms: (range_days > 0).then(|| to_ms - range_days * DAY_MS),
        to_ms,
    }
}

fn build_fresh(
    db: &DatabaseService,
    input: &ActivityPageBuildInput,
    range_days: i64,
    cursor: &str,
    window: &WindowBounds,
) -> Result<ActivityPageView, Error> {
    let to_ms = window.to_ms;
    let from_ms = window.from_ms;
    let previous_from_ms = from_ms.map(|from_ms| from_ms - range_days * DAY_MS);
    let window_spans = read_location_spans(db, &input.owner_user_id, from_ms, to_ms, input.now_ms)?;
    let spans = window_spans.spans;

    let previous = match (previous_from_ms, from_ms) {
        (Some(previous_from_ms), Some(from_ms)) => {
            let previous_spans = read_location_spans(
                db,
                &input.owner_user_id,
                Some(previous_from_ms),
                from_ms,
                input.now_ms,
            )?;
            summarize_previous(&previous_spans.spans, input.utc_offset_minutes)
        }
        _ => Default::default(),
    };

    let earlier_world_ids = match from_ms {
        Some(from_ms) => world_ids_before(db, &input.owner_user_id, from_ms)?,
        None => BTreeSet::new(),
    };

    let window_days = match range_days {
        0 => window_days_from_spans(&spans),
        days => days,
    };

    Ok(ActivityPageView {
        range_days,
        utc_offset_minutes: input.utc_offset_minutes,
        window_from_ms: from_ms.unwrap_or(0),
        window_to_ms: to_ms,
        has_open_tail: window_spans.has_open_tail,
        summary: summarize(&spans, window_days, input.utc_offset_minutes),
        previous,
        series: series(
            &spans,
            series_bucket_for_range(range_days),
            input.utc_offset_minutes,
        ),
        access_split: access_split(&spans),
        worlds: worlds(&spans, &earlier_world_ids, TOP_WORLD_LIMIT),
        people: people(
            db,
            &input.owner_user_id,
            from_ms,
            to_ms,
            input.utc_offset_minutes,
        )?,
        coverage: coverage(db, &input.owner_user_id, from_ms, to_ms)?,
        built_from_cursor: cursor.to_string(),
        built_at: activity_iso_from_ms(input.now_ms),
        stale: false,
    })
}

fn window_days_from_spans(spans: &[LocationSpan]) -> i64 {
    match (spans.first(), spans.last()) {
        (Some(first), Some(last)) => ((last.end_ms - first.start_ms) / DAY_MS) + 1,
        _ => 0,
    }
}

fn coverage(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
    to_ms: i64,
) -> Result<ActivityPageCoverage, Error> {
    let first_source_at = first_source_created_at(db, owner_user_id)?;
    let from = match from_ms {
        Some(from_ms) => activity_iso_from_ms(from_ms),
        None => first_source_at.clone(),
    };
    Ok(ActivityPageCoverage {
        from,
        to: activity_iso_from_ms(to_ms),
        first_source_at,
    })
}

fn first_source_created_at(db: &DatabaseService, owner_user_id: &OwnerId) -> Result<String, Error> {
    ensure_game_log_tables(db)?;
    Ok(db
        .execute(
            "SELECT MIN(created_at) FROM gamelog_location WHERE owner_id IN (0, @owner_id)",
            &ParamsBuilder::new()
                .set("owner_id", owner_id_for_filter(db, owner_user_id)?)
                .build(),
        )?
        .first()
        .map(|row| row_string(row, 0))
        .unwrap_or_default())
}

fn world_ids_before(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    before_ms: i64,
) -> Result<BTreeSet<String>, Error> {
    ensure_game_log_tables(db)?;
    Ok(db
        .execute(
            "SELECT DISTINCT world_id
             FROM gamelog_location
             WHERE owner_id IN (0, @owner_id)
               AND created_at < @before_iso
               AND world_id LIKE 'wrld_%'",
            &ParamsBuilder::new()
                .set("owner_id", owner_id_for_filter(db, owner_user_id)?)
                .set("before_iso", activity_iso_from_ms(before_ms))
                .build(),
        )?
        .into_iter()
        .map(|row| row_string(&row, 0))
        .filter(|world_id| !world_id.is_empty())
        .collect())
}
