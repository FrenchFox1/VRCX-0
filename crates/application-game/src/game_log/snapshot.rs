use serde::Serialize;

use vrcx_0_persistence::game_log::get_join_leave_entries_for_location_range;
use vrcx_0_persistence::player_list::{
    player_list_latest_location_get, player_list_location_get, PlayerLocationOutput,
};
use vrcx_0_persistence::DatabaseService;

use super::roster::fold_roster;
use super::runtime_state::{parse_event_time_ms, world_id_from_location};
use crate::Result;
use vrcx_0_persistence::OwnerId;

const ROSTER_RANGE_END: &str = "9999-12-31T23:59:59Z";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum PlayerListSnapshotSource {
    Database,
    None,
    Runtime,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerListSnapshotContext {
    pub created_at: String,
    pub location: String,
    pub world_id: String,
    pub world_name: String,
    pub time: i64,
    pub group_name: String,
    pub source: PlayerListSnapshotSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_player_event_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_facts_known: Option<bool>,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerListSnapshotPlayer {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub joined_at: String,
    pub joined_at_ms: i64,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerListSnapshotOutput {
    pub context: PlayerListSnapshotContext,
    pub players: Vec<PlayerListSnapshotPlayer>,
}

struct RosterRebuild {
    players: Vec<PlayerListSnapshotPlayer>,
    observed_player_event_count: i64,
}

fn parse_date_ms(value: &str) -> i64 {
    parse_event_time_ms(value.trim()).unwrap_or(0)
}

fn is_live_location(location: &str) -> bool {
    let normalized = location.trim();
    !normalized.is_empty()
        && normalized != "offline"
        && normalized != "private"
        && normalized != "traveling"
}

fn context_from_row(row: PlayerLocationOutput) -> PlayerListSnapshotContext {
    PlayerListSnapshotContext {
        created_at: row.created_at,
        location: row.location,
        world_id: row.world_id,
        world_name: row.world_name,
        time: row.time,
        group_name: row.group_name,
        source: PlayerListSnapshotSource::Database,
        player_count: None,
        observed_player_event_count: None,
        player_facts_known: None,
    }
}

fn empty_context(location: String, source: PlayerListSnapshotSource) -> PlayerListSnapshotContext {
    PlayerListSnapshotContext {
        created_at: String::new(),
        location,
        world_id: String::new(),
        world_name: String::new(),
        time: 0,
        group_name: String::new(),
        source,
        player_count: None,
        observed_player_event_count: None,
        player_facts_known: None,
    }
}

fn resolve_location_context(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    current_location: &str,
) -> Result<PlayerListSnapshotContext> {
    let normalized = current_location.trim().to_string();

    if is_live_location(&normalized) {
        if let Some(row) = player_list_location_get(db, owner_user_id, normalized.clone())? {
            return Ok(context_from_row(row));
        }
        let world_id = world_id_from_location(&normalized);
        let world_name = if world_id.is_empty() {
            normalized.clone()
        } else {
            world_id.clone()
        };
        let mut context = empty_context(normalized, PlayerListSnapshotSource::Runtime);
        context.world_id = world_id;
        context.world_name = world_name;
        return Ok(context);
    }

    if !normalized.is_empty() {
        return Ok(empty_context(normalized, PlayerListSnapshotSource::Runtime));
    }

    if let Some(row) = player_list_latest_location_get(db, owner_user_id)? {
        return Ok(context_from_row(row));
    }

    Ok(empty_context(String::new(), PlayerListSnapshotSource::None))
}

fn rebuild_roster(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    location: &str,
    started_at: &str,
    current_user_id: &str,
) -> Result<RosterRebuild> {
    let started_at = started_at.trim();
    let started_at_ms = parse_date_ms(started_at);
    let range_start = if started_at_ms > 0 { started_at } else { "" };
    let entries = get_join_leave_entries_for_location_range(
        db,
        owner_user_id,
        location.trim(),
        range_start,
        ROSTER_RANGE_END,
    )?;
    let entries = entries
        .into_iter()
        .filter(|entry| {
            if started_at_ms <= 0 {
                return true;
            }
            parse_event_time_ms(&entry.created_at).is_some_and(|event_ms| event_ms >= started_at_ms)
        })
        .collect::<Vec<_>>();
    let observed_player_event_count = entries.len() as i64;

    let mut players = fold_roster(&entries)
        .into_iter()
        .filter(|(_, player)| {
            current_user_id.is_empty() || player.user_id.trim() != current_user_id
        })
        .map(|(key, player)| {
            let display_name = if !player.display_name.is_empty() {
                player.display_name.clone()
            } else if !player.user_id.is_empty() {
                player.user_id.clone()
            } else {
                key.clone()
            };
            PlayerListSnapshotPlayer {
                id: key,
                user_id: player.user_id,
                display_name,
                joined_at: player.joined_at,
                joined_at_ms: player.joined_at_ms.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();
    players.sort_by(|left, right| {
        left.joined_at_ms
            .cmp(&right.joined_at_ms)
            .then_with(|| left.display_name.cmp(&right.display_name))
    });

    Ok(RosterRebuild {
        players,
        observed_player_event_count,
    })
}

pub fn player_list_current_snapshot(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    current_user_id: &str,
    current_location: &str,
    current_location_started_at: &str,
) -> Result<PlayerListSnapshotOutput> {
    let location_context = resolve_location_context(db, owner_user_id, current_location)?;

    let runtime_started_at = current_location_started_at.trim();
    let mut context = location_context.clone();
    if parse_date_ms(runtime_started_at) > parse_date_ms(&location_context.created_at) {
        context.created_at = runtime_started_at.to_string();
    }

    if !is_live_location(&context.location) {
        return Ok(PlayerListSnapshotOutput {
            context,
            players: Vec::new(),
        });
    }

    let current_user_id = current_user_id.trim();
    let mut roster = rebuild_roster(
        db,
        owner_user_id,
        &context.location,
        &context.created_at,
        current_user_id,
    )?;
    let mut effective_context = context.clone();

    let db_started_at_ms = parse_date_ms(&location_context.created_at);
    if roster.players.is_empty()
        && db_started_at_ms > 0
        && db_started_at_ms < parse_date_ms(&context.created_at)
    {
        let db_roster = rebuild_roster(
            db,
            owner_user_id,
            &location_context.location,
            &location_context.created_at,
            current_user_id,
        )?;
        if !db_roster.players.is_empty() {
            roster = db_roster;
            effective_context = location_context;
        }
    }

    effective_context.player_count = Some(roster.players.len() as i64);
    effective_context.observed_player_event_count = Some(roster.observed_player_event_count);
    effective_context.player_facts_known = Some(roster.observed_player_event_count > 0);

    Ok(PlayerListSnapshotOutput {
        context: effective_context,
        players: roster.players,
    })
}

#[cfg(test)]
mod tests;
