use std::path::PathBuf;

use vrcx_0_persistence::game_log::{
    write_batch, GameLogJoinLeaveEntry, GameLogLocationEntry, GameLogWriteBatch,
};

use crate::game_log::runtime_state::{PlayerState, RuntimeSnapshot};

use super::*;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("vrcx-0-{name}-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn test_db(name: &str) -> (TestDir, DatabaseService) {
    let dir = TestDir::new(name);
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap();
    (dir, db)
}

const LOCATION: &str = "wrld_live:123";
const LOCATION_AT: &str = "2026-04-30T10:00:00.000Z";
const SHOT_AT: &str = "2026-04-30T10:15:00.000Z";

fn join_leave_entry(
    created_at: &str,
    event_type: &str,
    display_name: &str,
    user_id: &str,
) -> GameLogJoinLeaveEntry {
    GameLogJoinLeaveEntry {
        created_at: created_at.to_string(),
        event_type: event_type.to_string(),
        display_name: display_name.to_string(),
        location: LOCATION.to_string(),
        user_id: user_id.to_string(),
        world_name: "Live World".to_string(),
        time: 0,
    }
}

fn write_visited_location(db: &DatabaseService, join_leave: Vec<GameLogJoinLeaveEntry>) {
    let batch = GameLogWriteBatch {
        locations: vec![GameLogLocationEntry {
            created_at: LOCATION_AT.to_string(),
            location: LOCATION.to_string(),
            world_id: "wrld_live".to_string(),
            world_name: "Live World".to_string(),
            time: 0,
            group_name: String::new(),
        }],
        join_leave,
        ..Default::default()
    };
    write_batch(db, &OwnerId::new(""), &batch).unwrap();
}

fn context_at(db: &DatabaseService, created_at: &str) -> Option<ScreenshotContext> {
    let input = ScreenshotInput {
        created_at: created_at.to_string(),
        path: "screenshot.png".to_string(),
        snapshot: RuntimeSnapshot::default(),
    };
    screenshot_context(db, &OwnerId::new(""), &input).unwrap()
}

#[test]
fn snapshot_location_short_circuits_the_database_lookup() {
    let (_dir, db) = test_db("screenshot-snapshot-shortcircuit");
    let input = ScreenshotInput {
        created_at: SHOT_AT.to_string(),
        path: "screenshot.png".to_string(),
        snapshot: RuntimeSnapshot {
            location: LOCATION.to_string(),
            world_name: "Live World".to_string(),
            destination: String::new(),
            started_at: String::new(),
            players: vec![PlayerState {
                user_id: "usr_a".to_string(),
                display_name: "Alice".to_string(),
                join_time_ms: None,
            }],
        },
    };

    let context = screenshot_context(&db, &OwnerId::new(""), &input)
        .unwrap()
        .unwrap();
    assert_eq!(context.location, LOCATION);
    assert_eq!(context.world_name, "Live World");
    assert_eq!(context.players.len(), 1);
    assert_eq!(context.players[0].user_id, "usr_a");
}

#[test]
fn no_location_history_returns_none() {
    let (_dir, db) = test_db("screenshot-no-history");

    assert!(context_at(&db, SHOT_AT).is_none());
}

#[test]
fn location_exactly_at_max_age_boundary_is_still_used() {
    let (_dir, db) = test_db("screenshot-boundary-included");
    write_visited_location(&db, vec![]);

    let context = context_at(&db, SHOT_AT).unwrap();

    assert_eq!(context.location, LOCATION);
    assert!(context.players.is_empty());
}

#[test]
fn location_one_ms_past_max_age_boundary_is_rejected() {
    let (_dir, db) = test_db("screenshot-boundary-excluded");
    write_visited_location(&db, vec![]);

    assert!(context_at(&db, "2026-04-30T10:15:00.001Z").is_none());
}

#[test]
fn join_events_dedupe_by_user_id_key() {
    let (_dir, db) = test_db("screenshot-join-dedupe-id");
    write_visited_location(
        &db,
        vec![
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Alice",
                "usr_a",
            ),
            join_leave_entry(
                "2026-04-30T10:02:00.000Z",
                "OnPlayerJoined",
                "Alice",
                "usr_a",
            ),
        ],
    );

    let context = context_at(&db, SHOT_AT).unwrap();

    assert_eq!(context.players.len(), 1);
    assert_eq!(context.players[0].user_id, "usr_a");
}

#[test]
fn join_events_dedupe_by_display_name_when_user_id_missing() {
    let (_dir, db) = test_db("screenshot-join-dedupe-name");
    write_visited_location(
        &db,
        vec![
            join_leave_entry("2026-04-30T10:01:00.000Z", "OnPlayerJoined", "NoId", ""),
            join_leave_entry("2026-04-30T10:02:00.000Z", "OnPlayerJoined", "NoId", ""),
        ],
    );

    let context = context_at(&db, SHOT_AT).unwrap();

    assert_eq!(context.players.len(), 1);
    assert_eq!(context.players[0].display_name, "NoId");
}

#[test]
fn distinct_user_ids_with_the_same_display_name_are_not_merged() {
    let (_dir, db) = test_db("screenshot-join-distinct-ids");
    write_visited_location(
        &db,
        vec![
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Twin",
                "usr_a",
            ),
            join_leave_entry(
                "2026-04-30T10:02:00.000Z",
                "OnPlayerJoined",
                "Twin",
                "usr_b",
            ),
        ],
    );

    let context = context_at(&db, SHOT_AT).unwrap();

    assert_eq!(context.players.len(), 2);
}

#[test]
fn leave_removes_player_by_user_id_key() {
    let (_dir, db) = test_db("screenshot-leave-by-id");
    write_visited_location(
        &db,
        vec![
            join_leave_entry(
                "2026-04-30T10:01:00.000Z",
                "OnPlayerJoined",
                "Alice",
                "usr_a",
            ),
            join_leave_entry("2026-04-30T10:02:00.000Z", "OnPlayerLeft", "Alice", "usr_a"),
        ],
    );

    let context = context_at(&db, SHOT_AT).unwrap();

    assert!(context.players.is_empty());
}

#[test]
fn leave_removes_anonymous_player_by_display_name_key() {
    let (_dir, db) = test_db("screenshot-leave-by-name");
    write_visited_location(
        &db,
        vec![
            join_leave_entry("2026-04-30T10:01:00.000Z", "OnPlayerJoined", "Bob", ""),
            join_leave_entry("2026-04-30T10:02:00.000Z", "OnPlayerLeft", "Bob", ""),
        ],
    );

    let context = context_at(&db, SHOT_AT).unwrap();

    assert!(context.players.is_empty());
}

#[test]
fn captured_author_does_not_follow_a_later_account_switch() {
    let scope = crate::RuntimeAuthScope::new();
    scope.set_identity("usr_first", "First User", "");
    let author = scope.identity();
    scope.set_identity("usr_second", "Second User", "");

    let metadata = build_metadata(&author, &ScreenshotContext::default(), "wrld_example");

    assert_eq!(metadata["author"]["id"], "usr_first");
    assert_eq!(metadata["author"]["displayName"], "First User");
}
