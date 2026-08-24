use std::collections::BTreeSet;

use crate::activity::activity_iso_from_ms;
use crate::common::{row_string, ParamsBuilder};
use crate::database::DatabaseService;
use crate::game_log::ensure_game_log_tables;
use crate::ownership::{owner_id_for_filter, OwnerId};
use crate::social_aggregates::{
    get_copresence_summary, get_fading_friends, CopresenceGroupBy, CopresenceOrderBy,
    CopresenceSummaryInput, FadingFriendsInput, TimeWindow,
};
use crate::Error;

use super::types::{ActivityPageCompanionRow, ActivityPageFadingRow, ActivityPagePeople};

const COMPANION_LIMIT: i64 = 10;
const FADING_LIMIT: i64 = 10;

pub(super) fn people(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
    to_ms: i64,
    utc_offset_minutes: i64,
) -> Result<ActivityPagePeople, Error> {
    let window = TimeWindow {
        from: from_ms.map(activity_iso_from_ms),
        to: Some(activity_iso_from_ms(to_ms)),
    };

    let encountered = encountered_user_ids(db, owner_user_id, from_ms, Some(to_ms))?;
    let previously_encountered = match from_ms {
        Some(from_ms) => encountered_user_ids(db, owner_user_id, None, Some(from_ms))?,
        None => BTreeSet::new(),
    };
    let new_face_count = encountered
        .iter()
        .filter(|user_id| !previously_encountered.contains(*user_id))
        .count();

    Ok(ActivityPagePeople {
        companions: companions(db, owner_user_id, &window, utc_offset_minutes)?,
        fading: fading(db, owner_user_id, from_ms, to_ms)?,
        encountered_count: i64::try_from(encountered.len()).unwrap_or(i64::MAX),
        new_face_count: i64::try_from(new_face_count).unwrap_or(i64::MAX),
    })
}

fn companions(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    window: &TimeWindow,
    utc_offset_minutes: i64,
) -> Result<Vec<ActivityPageCompanionRow>, Error> {
    let summary = get_copresence_summary(
        db,
        CopresenceSummaryInput {
            time_window: window.clone(),
            group_by: CopresenceGroupBy::Friend,
            order_by: CopresenceOrderBy::CoDays,
            min_minutes: None,
            limit: Some(COMPANION_LIMIT),
            owner_user_id: Some(owner_user_id.clone()),
            friends_only: false,
            utc_offset_minutes: Some(utc_offset_minutes),
        },
    )?;

    Ok(summary
        .rows
        .into_iter()
        .map(|row| ActivityPageCompanionRow {
            user_id: row.user_id,
            display_name: row.display_name,
            is_friend: row.is_friend,
            minutes: row.total_minutes,
            co_days: i64::try_from(row.co_days).unwrap_or(i64::MAX),
            instances: i64::try_from(row.instances).unwrap_or(i64::MAX),
            last_seen_together: row.last_seen_together,
        })
        .collect())
}

fn fading(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
    to_ms: i64,
) -> Result<Vec<ActivityPageFadingRow>, Error> {
    let Some(from_ms) = from_ms else {
        return Ok(Vec::new());
    };
    let prior_from_ms = from_ms - (to_ms - from_ms);
    let output = get_fading_friends(
        db,
        FadingFriendsInput {
            owner_user_id: owner_user_id.clone(),
            prior_from: activity_iso_from_ms(prior_from_ms),
            pivot: activity_iso_from_ms(from_ms),
            now: activity_iso_from_ms(to_ms),
            min_prior_minutes: None,
            limit: Some(FADING_LIMIT),
        },
    )?;
    Ok(output
        .rows
        .into_iter()
        .map(|row| ActivityPageFadingRow {
            user_id: row.user_id,
            display_name: row.display_name,
            prior_minutes: row.prior_minutes,
            recent_minutes: row.recent_minutes,
            drop_percent: row.drop_percent,
            last_seen_together: row.last_seen_together,
        })
        .collect())
}

fn encountered_user_ids(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
    to_ms: Option<i64>,
) -> Result<BTreeSet<String>, Error> {
    ensure_game_log_tables(db)?;
    let mut sql = String::from(
        "SELECT DISTINCT user_id
         FROM gamelog_join_leave
         WHERE owner_id IN (0, @owner_id)
           AND trim(user_id) <> ''
           AND user_id <> @owner_user_id",
    );
    let mut params = ParamsBuilder::new()
        .set("owner_id", owner_id_for_filter(db, owner_user_id)?)
        .set("owner_user_id", owner_user_id.as_str());
    if let Some(from_ms) = from_ms {
        sql.push_str(" AND created_at >= @from_iso");
        params = params.set("from_iso", activity_iso_from_ms(from_ms));
    }
    if let Some(to_ms) = to_ms {
        sql.push_str(" AND created_at < @to_iso");
        params = params.set("to_iso", activity_iso_from_ms(to_ms));
    }
    Ok(db
        .execute(&sql, &params.build())?
        .into_iter()
        .map(|row| row_string(&row, 0))
        .filter(|user_id| !user_id.is_empty())
        .collect())
}
