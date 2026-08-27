use crate::ports::TestGameStateStore;
use crate::GameStateStore;
use vrcx_0_contracts::game_log::{GameLogJoinLeaveEntry, GameLogLocationEntry, GameLogWriteBatch};

use super::*;

fn test_store(_name: &str) -> TestGameStateStore {
    TestGameStateStore::default()
}

fn location_entry(created_at: &str, location: &str) -> GameLogLocationEntry {
    GameLogLocationEntry {
        created_at: created_at.to_string(),
        location: location.to_string(),
        world_id: "wrld_live".to_string(),
        world_name: "Live World".to_string(),
        time: 0,
        group_name: String::new(),
    }
}

fn join_leave_entry(
    created_at: &str,
    event_type: &str,
    display_name: &str,
    location: &str,
    user_id: &str,
) -> GameLogJoinLeaveEntry {
    join_leave_entry_with_time(created_at, event_type, display_name, location, user_id, 0)
}

fn join_leave_entry_with_time(
    created_at: &str,
    event_type: &str,
    display_name: &str,
    location: &str,
    user_id: &str,
    time: i64,
) -> GameLogJoinLeaveEntry {
    GameLogJoinLeaveEntry {
        created_at: created_at.to_string(),
        event_type: event_type.to_string(),
        display_name: display_name.to_string(),
        location: location.to_string(),
        user_id: user_id.to_string(),
        world_name: "Live World".to_string(),
        time,
    }
}

fn write_rows(
    store: &TestGameStateStore,
    locations: Vec<GameLogLocationEntry>,
    join_leave: Vec<GameLogJoinLeaveEntry>,
) {
    let batch = GameLogWriteBatch {
        locations,
        join_leave,
        ..Default::default()
    };
    store.write_game_log(&OwnerId::new(""), &batch).unwrap();
}

#[test]
fn excludes_join_rows_from_earlier_visits_to_the_same_instance() {
    let store = test_store("snapshot-earlier-visits");
    write_rows(
        &store,
        vec![location_entry("2026-04-30T10:00:00.000Z", "wrld_live:123")],
        vec![
            join_leave_entry(
                "2026-01-01T10:00:00.000Z",
                "OnPlayerJoined",
                "Old Player",
                "wrld_live:123",
                "usr_old",
            ),
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Current Player",
                "wrld_live:123",
                "usr_current",
            ),
        ],
    );

    let snapshot =
        player_list_current_snapshot(&store, &OwnerId::new(""), "", "wrld_live:123", "").unwrap();
    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].user_id, "usr_current");
    assert_eq!(snapshot.context.player_count, Some(1));
}

#[test]
fn runtime_start_time_overrides_stale_database_location_rows() {
    let store = test_store("snapshot-runtime-start");
    write_rows(
        &store,
        vec![location_entry("2026-01-01T10:00:00.000Z", "wrld_live:123")],
        vec![
            join_leave_entry(
                "2026-01-01T10:01:00.000Z",
                "OnPlayerJoined",
                "Old Player",
                "wrld_live:123",
                "usr_old",
            ),
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Current Player",
                "wrld_live:123",
                "usr_current",
            ),
        ],
    );

    let snapshot = player_list_current_snapshot(
        &store,
        &OwnerId::new(""),
        "",
        "wrld_live:123",
        "2026-04-30T10:00:00.000Z",
    )
    .unwrap();
    assert_eq!(snapshot.context.created_at, "2026-04-30T10:00:00.000Z");
    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].user_id, "usr_current");
}

#[test]
fn leave_with_id_removes_unique_anonymous_join_by_display_name() {
    let store = test_store("snapshot-anonymous-leave");
    write_rows(
        &store,
        vec![location_entry("2026-04-30T10:00:00.000Z", "wrld_live:123")],
        vec![
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Left Player",
                "wrld_live:123",
                "",
            ),
            join_leave_entry(
                "2026-04-30T10:02:00.000Z",
                "OnPlayerLeft",
                "Left Player",
                "wrld_live:123",
                "usr_left",
            ),
        ],
    );

    let snapshot =
        player_list_current_snapshot(&store, &OwnerId::new(""), "", "wrld_live:123", "").unwrap();
    assert!(snapshot.players.is_empty());
}

#[test]
fn anonymous_leave_uses_duration_when_display_name_is_ambiguous() {
    let store = test_store("snapshot-anonymous-duration-leave");
    write_rows(
        &store,
        vec![location_entry("2026-04-30T10:00:00.000Z", "wrld_live:123")],
        vec![
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Guest",
                "wrld_live:123",
                "",
            ),
            join_leave_entry(
                "2026-04-30T10:01:30.000Z",
                "OnPlayerJoined",
                "Guest",
                "wrld_live:123",
                "",
            ),
            join_leave_entry_with_time(
                "2026-04-30T10:02:00.000Z",
                "OnPlayerLeft",
                "Guest",
                "wrld_live:123",
                "",
                60_000,
            ),
        ],
    );

    let snapshot =
        player_list_current_snapshot(&store, &OwnerId::new(""), "", "wrld_live:123", "").unwrap();
    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].display_name, "Guest");
    assert_eq!(snapshot.players[0].joined_at, "2026-04-30T10:01:30.000Z");
}

#[test]
fn falls_back_to_database_enter_time_when_stale_runtime_start_empties_roster() {
    let store = test_store("snapshot-db-window-fallback");
    write_rows(
        &store,
        vec![location_entry(
            "2026-06-09T12:26:31.000Z",
            "wrld_live:83220",
        )],
        vec![join_leave_entry(
            "2026-06-09T12:26:59.000Z",
            "OnPlayerJoined",
            "CyanChanges",
            "wrld_live:83220",
            "usr_cyan",
        )],
    );

    let snapshot = player_list_current_snapshot(
        &store,
        &OwnerId::new(""),
        "",
        "wrld_live:83220",
        "2026-06-10T19:00:00.000Z",
    )
    .unwrap();
    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].user_id, "usr_cyan");
    assert_eq!(snapshot.context.created_at, "2026-06-09T12:26:31.000Z");
    assert_eq!(snapshot.context.player_facts_known, Some(true));
}

#[test]
fn current_user_filter_can_empty_roster_and_trigger_facts_known() {
    let store = test_store("snapshot-current-user-filter");
    write_rows(
        &store,
        vec![location_entry("2026-04-30T10:00:00.000Z", "wrld_live:123")],
        vec![join_leave_entry(
            "2026-04-30T10:01:00.000Z",
            "OnPlayerJoined",
            "Me",
            "wrld_live:123",
            "usr_me",
        )],
    );

    let snapshot = player_list_current_snapshot(
        &store,
        &OwnerId::new("usr_me"),
        "usr_me",
        "wrld_live:123",
        "",
    )
    .unwrap();
    assert!(snapshot.players.is_empty());
    assert_eq!(snapshot.context.player_count, Some(0));
    assert_eq!(snapshot.context.player_facts_known, Some(true));
}

#[test]
fn non_live_location_returns_context_without_roster() {
    let store = test_store("snapshot-non-live");
    let snapshot =
        player_list_current_snapshot(&store, &OwnerId::new(""), "", "private", "").unwrap();
    assert_eq!(snapshot.context.source, PlayerListSnapshotSource::Runtime);
    assert_eq!(snapshot.context.location, "private");
    assert!(snapshot.players.is_empty());
    assert_eq!(snapshot.context.player_count, None);
}
