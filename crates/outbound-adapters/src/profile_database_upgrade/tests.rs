use std::path::PathBuf;

use super::*;
use vrcx_0_persistence::migration::migration_version;

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

    fn database(&self) -> DatabaseService {
        DatabaseService::new(&self.path.join("VRCX-0.sqlite3")).unwrap()
    }
}

#[test]
fn stage_label_matches_the_ipc_camel_case_name() {
    assert_eq!(stage_label(DatabaseUpgradeStage::Preflight), "preflight");
    assert_eq!(
        stage_label(DatabaseUpgradeStage::LegacySchemaMigration),
        "legacySchemaMigration"
    );
    assert_eq!(stage_label(DatabaseUpgradeStage::Commit), "commit");
}

#[test]
fn sqlite_categories_use_stable_telemetry_labels() {
    for (category, expected) in [
        (Some(SqliteErrorCategory::Malformed), "malformed"),
        (Some(SqliteErrorCategory::DiskFull), "disk_full"),
        (Some(SqliteErrorCategory::Locked), "locked"),
        (Some(SqliteErrorCategory::IoError), "io_error"),
        (None, "unclassified"),
    ] {
        let error = Error::Sqlite {
            message: "injected SQLite failure".into(),
            category,
        };
        assert_eq!(sqlite_category_label(&error), expected);
    }
    assert_eq!(
        sqlite_category_label(&Error::Custom("injected non-SQLite failure".into())),
        "none"
    );
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn set_version(db: &DatabaseService, version: i64) {
    write_database_schema_versions(db, version).unwrap();
}

fn set_migration_version(db: &DatabaseService, version: i64) {
    rusqlite::Connection::open(db.db_path())
        .unwrap()
        .execute_batch(&format!("PRAGMA user_version = {version}"))
        .unwrap();
}

fn install_failing_repair_fixture(db: &DatabaseService) {
    database_maintenance_run(db, DatabaseMaintenanceTask::InitGlobalTables).unwrap();
    let conn = rusqlite::Connection::open(db.db_path()).unwrap();
    conn.execute_batch(
        "INSERT INTO gamelog_join_leave
                 (created_at, type, display_name, location, user_id, time)
             VALUES
                 ('2026-01-01T00:00:00Z', 'OnPlayerJoined', 'Test', 'wrld_test:1', 'usr_test', 0),
                 ('2026-01-01T00:01:00Z', 'OnPlayerLeft', 'Test', 'wrld_test:1', 'usr_test', 0);
             CREATE TRIGGER reject_copresence_repair
             BEFORE UPDATE OF time ON gamelog_join_leave
             BEGIN
                 SELECT RAISE(FAIL, 'repair should not run');
             END;",
    )
    .unwrap();
}

#[test]
fn preflight_reports_current_upgrade_and_newer_spans() {
    use DatabaseUpgradePreflightStatus::{Current, NewerSchema, UpgradeRequired};

    let target = target_migration_version();
    let schema = VRCX0_SCHEMA_VERSION;
    for (schema_version, migration, expected, span) in [
        (0, target, UpgradeRequired, (target, target)),
        (16, target, UpgradeRequired, (target, target)),
        (schema, target, Current, (target, target)),
        (schema + 1, target, NewerSchema, (schema + 1, schema)),
        (schema, target + 1, NewerSchema, (target + 1, target)),
    ] {
        let dir = TestDir::new(&format!(
            "database-upgrade-preflight-{schema_version}-{migration}"
        ));
        let db = dir.database();
        if schema_version > 0 {
            set_version(&db, schema_version);
        }
        set_migration_version(&db, migration);

        let preflight = database_upgrade_preflight(&db).unwrap();

        assert_eq!(preflight.status, expected);
        assert_eq!((preflight.from_version, preflight.to_version), span);
    }
}

#[test]
fn upgrades_every_supported_old_version_span_and_is_idempotent() {
    for version in [0, 15, 16, 17] {
        let dir = TestDir::new(&format!("database-upgrade-span-{version}"));
        let db = dir.database();
        if version > 0 {
            set_version(&db, version);
        }

        let upgraded = run_database_upgrade(&db);

        assert_eq!(upgraded.status, DatabaseUpgradeRunStatus::Upgraded);
        assert_eq!(upgraded.from_version, 0);
        assert_eq!(upgraded.to_version, target_migration_version());
        assert_eq!(migration_version(&db).unwrap(), target_migration_version());
        assert_eq!(
            prepare_vrcx0_schema_version(&db).unwrap(),
            VRCX0_SCHEMA_VERSION
        );
        assert_eq!(
            vrcx_0_persistence::config::get_string(&db, "databaseVersion", "0").unwrap(),
            VRCX0_SCHEMA_VERSION.to_string()
        );
        assert_eq!(
            vrcx_0_persistence::config::get_string(&db, COPRESENCE_DURATION_REPAIR_KEY, "")
                .unwrap(),
            "1"
        );

        let repeated = run_database_upgrade(&db);
        assert_eq!(repeated.status, DatabaseUpgradeRunStatus::Current);
        assert_eq!(repeated.from_version, target_migration_version());
    }
}

#[test]
fn reports_determinate_work_copy_progress_and_indeterminate_schema_stages() {
    let dir = TestDir::new("database-upgrade-progress");
    let db = dir.database();
    set_version(&db, 17);
    rusqlite::Connection::open(db.db_path())
        .unwrap()
        .execute_batch(
            "CREATE TABLE progress_fixture (payload BLOB);
                 INSERT INTO progress_fixture (payload) VALUES (zeroblob(2097152));",
        )
        .unwrap();
    let mut progress = Vec::new();

    let result = run_database_upgrade_with_progress(&db, |snapshot| progress.push(snapshot));

    assert_eq!(result.status, DatabaseUpgradeRunStatus::Upgraded);
    assert!(progress.iter().any(|snapshot| {
        snapshot.stage == DatabaseUpgradeStage::CreateWorkCopy
            && snapshot.total_units.is_some_and(|total| total > 0)
            && snapshot.completed_units == snapshot.total_units
    }));
    assert!(progress.iter().any(|snapshot| {
        snapshot.stage == DatabaseUpgradeStage::NotificationPerformanceIndexes
            && snapshot.total_units.is_none()
    }));
}

#[test]
fn legacy_upgrade_repairs_rows_that_match_old_cleanup_rules() {
    let dir = TestDir::new("database-upgrade-repairs-legacy-rows");
    let db = dir.database();
    set_version(&db, 15);
    rusqlite::Connection::open(db.db_path())
        .unwrap()
        .execute_batch(
            "CREATE TABLE usr_test_friend_log_history (
                id INTEGER PRIMARY KEY,
                created_at TEXT,
                type TEXT,
                trust_level TEXT,
                previous_trust_level TEXT,
                user_id TEXT
            );
             INSERT INTO usr_test_friend_log_history
                (created_at, type, trust_level, previous_trust_level, user_id)
             VALUES
                ('2026-01-01T00:00:00Z', 'TrustLevel', 'Veteran User', 'Trusted User', 'usr_glitch'),
                ('2026-01-01T00:00:00Z', 'CancelFriendRequst', NULL, NULL, 'usr_typo');",
        )
        .unwrap();

    let result = run_database_upgrade(&db);

    assert_eq!(result.status, DatabaseUpgradeRunStatus::Upgraded);
    let conn = rusqlite::Connection::open(db.db_path()).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM usr_test_friend_log_history WHERE user_id = 'usr_glitch'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT type FROM usr_test_friend_log_history WHERE user_id = 'usr_typo'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        "CancelFriendRequest"
    );
}

#[test]
fn preserves_failed_work_copy_and_blocks_reentry() {
    let dir = TestDir::new("database-upgrade-failed-copy");
    let db = dir.database();
    set_version(&db, 17);
    vrcx_0_persistence::game_log::ensure_game_log_tables(&db).unwrap();
    let conn = rusqlite::Connection::open(db.db_path()).unwrap();
    conn.execute_batch(
        "DROP TABLE gamelog_location;
             CREATE TABLE gamelog_location (id INTEGER PRIMARY KEY);",
    )
    .unwrap();
    drop(conn);

    let failed = run_database_upgrade(&db);

    assert_eq!(failed.status, DatabaseUpgradeRunStatus::Failed);
    assert_eq!(
        failed.failed_stage,
        Some(DatabaseUpgradeStage::LegacyPerformanceIndexes)
    );
    let error = failed.error.as_deref().expect("database upgrade error");
    assert!(error.contains("no such column: created_at"));
    assert!(error.contains(
        "CREATE INDEX IF NOT EXISTS gamelog_location_created_at_idx ON gamelog_location (created_at)"
    ));
    let failed_upgrade = failed.failed_upgrade.expect("failed upgrade status");
    assert_eq!(
        failed_upgrade.operation.as_deref(),
        Some("database_maintenance_run:addLegacyPerformanceIndexes")
    );
    assert!(std::path::Path::new(&failed_upgrade.work_db_path).exists());
    assert!(db.is_main_mode());
    assert_eq!(prepare_vrcx0_schema_version(&db).unwrap(), 17);

    let blocked = run_database_upgrade(&db);
    assert_eq!(blocked.status, DatabaseUpgradeRunStatus::Blocked);
    assert_eq!(blocked.from_version, 17);
    assert_eq!(blocked.to_version, VRCX0_SCHEMA_VERSION);
}

#[test]
fn refuses_to_modify_a_newer_schema() {
    let dir = TestDir::new("database-upgrade-newer-schema");
    let db = dir.database();
    set_version(&db, VRCX0_SCHEMA_VERSION + 1);

    let result = run_database_upgrade(&db);

    assert_eq!(result.status, DatabaseUpgradeRunStatus::NewerSchema);
    assert_eq!(
        prepare_vrcx0_schema_version(&db).unwrap(),
        VRCX0_SCHEMA_VERSION + 1
    );
    assert!(db.get_failed_upgrade().unwrap().is_none());
}

#[test]
fn one_time_repair_uses_its_own_marker_and_retries_non_fatal_failures() {
    let skipped_dir = TestDir::new("database-upgrade-repair-skipped");
    let skipped_db = skipped_dir.database();
    set_version(&skipped_db, VRCX0_SCHEMA_VERSION);
    set_migration_version(&skipped_db, target_migration_version());
    install_failing_repair_fixture(&skipped_db);
    vrcx_0_persistence::config::set_string(&skipped_db, COPRESENCE_DURATION_REPAIR_KEY, "1")
        .unwrap();

    let skipped = run_database_upgrade(&skipped_db);

    assert_eq!(skipped.status, DatabaseUpgradeRunStatus::Current);
    assert!(skipped.repair_warning.is_none());

    let retry_dir = TestDir::new("database-upgrade-repair-retry");
    let retry_db = retry_dir.database();
    set_version(&retry_db, VRCX0_SCHEMA_VERSION);
    set_migration_version(&retry_db, target_migration_version());
    install_failing_repair_fixture(&retry_db);

    let retry = run_database_upgrade(&retry_db);

    assert_eq!(retry.status, DatabaseUpgradeRunStatus::Current);
    assert!(retry
        .repair_warning
        .as_deref()
        .is_some_and(|warning| warning.contains("repair should not run")));
    assert_eq!(
        vrcx_0_persistence::config::get_string(&retry_db, COPRESENCE_DURATION_REPAIR_KEY, "")
            .unwrap(),
        ""
    );
}
