use std::collections::HashMap;

use serde_json::Value;
use vrcx_0_core::activity_sessions::{span_duration_ms, SpanEnd};

use crate::activity::parse_activity_time_ms;
use crate::common::{row_i64, row_string};
use crate::database::DatabaseService;
use crate::game_log::ensure_game_log_tables;
use crate::ownership::{owner_id_for_filter, OwnerId};
use crate::social_aggregates::{access_bucket_sql, world_id_from_location_sql};
use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LocationSpan {
    pub(super) start_ms: i64,
    pub(super) end_ms: i64,
    pub(super) world_id: String,
    pub(super) world_name: String,
    pub(super) access_bucket: String,
    pub(super) inferred: bool,
}

impl LocationSpan {
    pub(super) fn duration_ms(&self) -> i64 {
        self.end_ms - self.start_ms
    }
}

struct SourceRow {
    created_at: String,
    time: i64,
    world_id: String,
    world_name: String,
    access_bucket: String,
}

pub(super) struct WindowSpans {
    pub(super) spans: Vec<LocationSpan>,
    pub(super) has_open_tail: bool,
}

pub(super) fn read_location_spans(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
    to_ms: i64,
    now_ms: i64,
) -> Result<WindowSpans, Error> {
    let rows = read_source_rows(db, owner_user_id, from_ms)?;
    let has_open_tail = rows.last().is_some_and(|row| row.time == 0);
    Ok(WindowSpans {
        spans: clip_spans(&spans_from_rows(&rows, now_ms), from_ms, to_ms),
        has_open_tail,
    })
}

fn read_source_rows(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
) -> Result<Vec<SourceRow>, Error> {
    ensure_game_log_tables(db)?;
    let owner_id = owner_id_for_filter(db, owner_user_id)?;
    let world_id_expr = world_id_from_location_sql("location");
    let access_expr = access_bucket_sql("location");
    let mut params: HashMap<String, Value> = HashMap::new();
    params.insert("@owner_id".into(), Value::from(owner_id));

    let sql = match from_ms {
        Some(from_ms) => {
            params.insert(
                "@from_iso".into(),
                Value::String(crate::activity::activity_iso_from_ms(from_ms)),
            );
            format!(
                "SELECT created_at, time, CASE WHEN world_id LIKE 'wrld_%' THEN world_id ELSE {world_id_expr} END AS world_id, world_name, {access_expr} AS access_bucket, sort_group, id
                 FROM (
                     SELECT created_at, time, location, world_id, world_name, id, 0 AS sort_group
                     FROM (
                         SELECT created_at, time, location, world_id, world_name, id
                         FROM gamelog_location
                         WHERE owner_id IN (0, @owner_id)
                           AND created_at < @from_iso
                         ORDER BY created_at DESC, id DESC
                         LIMIT 1
                     )
                     UNION ALL
                     SELECT created_at, time, location, world_id, world_name, id, 1 AS sort_group
                     FROM gamelog_location
                     WHERE owner_id IN (0, @owner_id)
                       AND created_at >= @from_iso
                 )
                 ORDER BY created_at ASC, sort_group ASC, id ASC"
            )
        }
        None => format!(
            "SELECT created_at, time, CASE WHEN world_id LIKE 'wrld_%' THEN world_id ELSE {world_id_expr} END AS world_id, world_name, {access_expr} AS access_bucket
             FROM gamelog_location
             WHERE owner_id IN (0, @owner_id)
             ORDER BY created_at ASC, id ASC"
        ),
    };

    Ok(db
        .execute(&sql, &params)?
        .into_iter()
        .map(|row| SourceRow {
            created_at: row_string(&row, 0),
            time: row_i64(&row, 1),
            world_id: row_string(&row, 2),
            world_name: row_string(&row, 3),
            access_bucket: row_string(&row, 4),
        })
        .collect())
}

fn spans_from_rows(rows: &[SourceRow], now_ms: i64) -> Vec<LocationSpan> {
    let mut spans = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let Some(start_ms) = parse_activity_time_ms(&row.created_at) else {
            continue;
        };
        let end = match rows.get(index + 1) {
            Some(next) => match parse_activity_time_ms(&next.created_at) {
                Some(next_start_ms) => SpanEnd::NextStart(next_start_ms),
                None => SpanEnd::UnknownNextStart,
            },
            None => SpanEnd::OpenTail,
        };
        let duration_ms = span_duration_ms(start_ms, row.time, end, now_ms);
        if duration_ms <= 0 {
            continue;
        }
        spans.push(LocationSpan {
            start_ms,
            end_ms: start_ms + duration_ms,
            world_id: row.world_id.clone(),
            world_name: row.world_name.clone(),
            access_bucket: row.access_bucket.clone(),
            inferred: row.time == 0,
        });
    }
    spans
}

fn clip_spans(spans: &[LocationSpan], from_ms: Option<i64>, to_ms: i64) -> Vec<LocationSpan> {
    let mut clipped = Vec::with_capacity(spans.len());
    for span in spans {
        let start_ms = match from_ms {
            Some(from_ms) => span.start_ms.max(from_ms),
            None => span.start_ms,
        };
        let end_ms = span.end_ms.min(to_ms);
        if end_ms <= start_ms {
            continue;
        }
        clipped.push(LocationSpan {
            start_ms,
            end_ms,
            world_id: span.world_id.clone(),
            world_name: span.world_name.clone(),
            access_bucket: span.access_bucket.clone(),
            inferred: span.inferred,
        });
    }
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: i64 = 1_700_000_000_000;
    const HOUR: i64 = 3_600_000;

    fn source_row(offset_ms: i64, time: i64) -> SourceRow {
        SourceRow {
            created_at: crate::activity::activity_iso_from_ms(BASE + offset_ms),
            time,
            world_id: "wrld_a".into(),
            world_name: "Alpha".into(),
            access_bucket: "public".into(),
        }
    }

    #[test]
    fn spans_use_recorded_time_when_present() {
        let spans = spans_from_rows(&[source_row(0, HOUR)], BASE + 10 * HOUR);

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].duration_ms(), HOUR);
    }

    #[test]
    fn spans_infer_missing_time_from_next_row() {
        let spans = spans_from_rows(
            &[source_row(0, 0), source_row(2 * HOUR, HOUR)],
            BASE + 10 * HOUR,
        );

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].duration_ms(), 2 * HOUR);
        assert_eq!(spans[1].duration_ms(), HOUR);
    }

    #[test]
    fn spans_infer_open_tail_from_now() {
        let spans = spans_from_rows(&[source_row(0, 0)], BASE + 3 * HOUR);

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].duration_ms(), 3 * HOUR);
    }

    #[test]
    fn clip_trims_spans_to_window_bounds() {
        let spans = vec![LocationSpan {
            start_ms: BASE,
            end_ms: BASE + 10 * HOUR,
            world_id: "wrld_a".into(),
            world_name: "Alpha".into(),
            access_bucket: "public".into(),
            inferred: false,
        }];

        let clipped = clip_spans(&spans, Some(BASE + 2 * HOUR), BASE + 5 * HOUR);

        assert_eq!(clipped.len(), 1);
        assert_eq!(clipped[0].start_ms, BASE + 2 * HOUR);
        assert_eq!(clipped[0].end_ms, BASE + 5 * HOUR);
    }

    #[test]
    fn clip_drops_spans_outside_window() {
        let spans = vec![LocationSpan {
            start_ms: BASE,
            end_ms: BASE + HOUR,
            world_id: "wrld_a".into(),
            world_name: "Alpha".into(),
            access_bucket: "public".into(),
            inferred: false,
        }];

        assert!(clip_spans(&spans, Some(BASE + 2 * HOUR), BASE + 5 * HOUR).is_empty());
    }
}
