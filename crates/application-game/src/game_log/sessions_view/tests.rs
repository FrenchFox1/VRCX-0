use std::path::PathBuf;
use std::sync::Arc;

use vrcx_0_persistence::game_log::{
    write_batch, GameLogJoinLeaveEntry, GameLogLocationEntry, GameLogVideoPlayEntry,
    GameLogWriteBatch,
};

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

fn test_db(name: &str) -> (TestDir, Arc<DatabaseService>) {
    let dir = TestDir::new(name);
    let db = Arc::new(DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap());
    (dir, db)
}

fn location(
    created_at: &str,
    location: &str,
    world_id: &str,
    world_name: &str,
) -> GameLogLocationEntry {
    GameLogLocationEntry {
        created_at: created_at.to_string(),
        location: location.to_string(),
        world_id: world_id.to_string(),
        world_name: world_name.to_string(),
        time: 0,
        group_name: String::new(),
    }
}

fn join(
    created_at: &str,
    display_name: &str,
    location: &str,
    user_id: &str,
) -> GameLogJoinLeaveEntry {
    GameLogJoinLeaveEntry {
        created_at: created_at.to_string(),
        event_type: "OnPlayerJoined".to_string(),
        display_name: display_name.to_string(),
        location: location.to_string(),
        user_id: user_id.to_string(),
        world_name: String::new(),
        time: 0,
    }
}

fn video(created_at: &str, url: &str, location: &str) -> GameLogVideoPlayEntry {
    GameLogVideoPlayEntry {
        created_at: created_at.to_string(),
        video_url: url.to_string(),
        video_name: "Clip".to_string(),
        video_id: String::new(),
        location: location.to_string(),
        display_name: String::new(),
        user_id: String::new(),
    }
}

fn write_rows(
    db: &DatabaseService,
    locations: Vec<GameLogLocationEntry>,
    join_leave: Vec<GameLogJoinLeaveEntry>,
    video_plays: Vec<GameLogVideoPlayEntry>,
) {
    let batch = GameLogWriteBatch {
        locations,
        join_leave,
        video_plays,
        ..Default::default()
    };
    write_batch(db, "", &batch).unwrap();
}

fn query(db: &DatabaseService, input: GameLogSessionsQueryInput) -> Vec<GameLogSessionDto> {
    game_log_sessions_query(db, "", input).unwrap()
}

#[test]
fn returns_sessions_newest_first_with_video_merge() {
    let (_dir, db) = test_db("sessions-newest-first");
    write_rows(
        &db,
        vec![
            location("2026-01-01T10:00:00.000Z", "wrld_old:1", "wrld_old", "Old"),
            location("2026-01-01T11:00:00.000Z", "wrld_new:1", "wrld_new", "New"),
        ],
        vec![join("2026-01-01T10:00:01.000Z", "A", "wrld_old:1", "usr_a")],
        vec![
            video("2026-01-01T11:00:01.000Z", "https://v.test/a", "wrld_new:1"),
            video("2026-01-01T11:00:02.000Z", "https://v.test/a", "wrld_new:1"),
        ],
    );

    let sessions = query(&db, GameLogSessionsQueryInput::default());

    assert_eq!(
        sessions
            .iter()
            .map(|s| s.world_id.as_str())
            .collect::<Vec<_>>(),
        vec!["wrld_new", "wrld_old"]
    );
    assert_eq!(sessions[0].events.len(), 1);
    assert_eq!(sessions[0].events[0].type_, "VideoPlay");
    assert_eq!(sessions[0].events[0].play_count, Some(2));
    assert_eq!(sessions[1].events[0].user_id.as_deref(), Some("usr_a"));
}

#[test]
fn filters_sessions_by_favorite_user() {
    let (_dir, db) = test_db("sessions-favorite");
    write_rows(
        &db,
        vec![
            location("2026-01-01T10:00:00.000Z", "wrld_a:1", "wrld_a", "A"),
            location("2026-01-01T11:00:00.000Z", "wrld_b:1", "wrld_b", "B"),
        ],
        vec![
            join("2026-01-01T10:00:01.000Z", "A", "wrld_a:1", "usr_a"),
            join("2026-01-01T11:00:01.000Z", "B", "wrld_b:1", "usr_b"),
        ],
        Vec::new(),
    );

    let sessions = query(
        &db,
        GameLogSessionsQueryInput {
            favorite_user_ids: vec!["usr_b".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].world_id, "wrld_b");
    assert_eq!(sessions[0].events[0].is_favorite, Some(true));
}

#[test]
fn global_search_matches_world_name_header() {
    let (_dir, db) = test_db("sessions-search");
    write_rows(
        &db,
        vec![
            location(
                "2026-01-01T10:00:00.000Z",
                "wrld_a:1",
                "wrld_a",
                "Alpha World",
            ),
            location(
                "2026-01-01T11:00:00.000Z",
                "wrld_b:1",
                "wrld_b",
                "Beta World",
            ),
        ],
        vec![
            join("2026-01-01T10:00:01.000Z", "A", "wrld_a:1", "usr_a"),
            join("2026-01-01T11:00:01.000Z", "B", "wrld_b:1", "usr_b"),
        ],
        Vec::new(),
    );

    let sessions = query(
        &db,
        GameLogSessionsQueryInput {
            search: "alpha".to_string(),
            ..Default::default()
        },
    );

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].world_name, "Alpha World");
}
