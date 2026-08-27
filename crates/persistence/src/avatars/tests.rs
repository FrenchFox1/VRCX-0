use std::path::PathBuf;

use serde_json::json;
use vrcx_0_core::ReleaseStatus;

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
        let path = std::env::temp_dir().join(format!(
            "vrcx-0-avatars-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn avatar_entry(id: &str) -> CacheEntityInput {
    CacheEntityInput {
        id: json!(id),
        author_id: json!("usr_author"),
        author_name: json!("Author"),
        created_at: json!("2026-01-01T00:00:00Z"),
        description: json!("Description"),
        image_url: json!("https://example.com/avatar.png"),
        name: json!("Shared Avatar"),
        release_status: json!("public"),
        thumbnail_image_url: json!("https://example.com/avatar-thumb.png"),
        updated_at: json!("2026-01-02T00:00:00Z"),
        version: json!(1),
    }
}

#[test]
fn clearing_one_accounts_history_preserves_other_accounts_history_and_global_cache(
) -> Result<(), Error> {
    let dir = TestDir::new("history-owner-isolation");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    let avatar_id = "avtr_shared";

    avatar_cache_upsert(&db, avatar_entry(avatar_id))?;
    avatar_time_spent_add(&db, "usr_a".into(), avatar_id.into(), 0)?;
    avatar_time_spent_add(&db, "usr_b".into(), avatar_id.into(), 42)?;

    assert_eq!(avatar_history_list(&db, "usr_a".into(), 100)?.len(), 1);
    assert_eq!(avatar_history_list(&db, "usr_b".into(), 100)?.len(), 1);

    avatar_history_clear(&db, "usr_a".into())?;

    assert!(avatar_history_list(&db, "usr_a".into(), 100)?.is_empty());
    assert_eq!(avatar_history_list(&db, "usr_b".into(), 100)?.len(), 1);
    assert_eq!(
        avatar_time_spent_get(&db, "usr_b".into(), avatar_id.into())?.time_spent,
        42
    );
    assert!(avatar_cache_get(&db, avatar_id.into())?.is_some());
    Ok(())
}

#[test]
fn cache_upsert_applies_the_shared_entity_id_invariant_to_avatars() -> Result<(), Error> {
    let dir = TestDir::new("normalized-cache-id");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;

    avatar_cache_upsert(&db, avatar_entry("  avtr_spaced  "))?;

    let cached = avatar_cache_get(&db, "avtr_spaced".into())?
        .expect("normalized avatar cache id should be readable");
    assert_eq!(cached.id, "avtr_spaced");
    Ok(())
}

#[test]
fn cache_lookup_by_file_id_returns_only_the_matching_avatar() -> Result<(), Error> {
    let dir = TestDir::new("cache-file-id-lookup");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    let mut first = avatar_entry("avtr_first");
    first.image_url = json!("https://api.vrchat.cloud/api/1/file/file_first/1/file");
    let mut second = avatar_entry("avtr_second");
    second.thumbnail_image_url = json!("https://api.vrchat.cloud/api/1/image/file_second/2/256");

    avatar_cache_upsert(&db, first)?;
    avatar_cache_upsert(&db, second)?;

    let cached = avatar_cache_find_by_file_id(&db, "file_second")?
        .expect("matching avatar should be returned");

    assert_eq!(cached.id, "avtr_second");
    assert!(avatar_cache_find_by_file_id(&db, "file_missing")?.is_none());
    Ok(())
}

#[test]
fn avatar_cache_preserves_unknown_release_status() -> Result<(), Error> {
    let dir = TestDir::new("unknown-release-status");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    let mut entry = avatar_entry("avtr_future");
    entry.release_status = json!("future");

    avatar_cache_upsert(&db, entry)?;

    let cached =
        avatar_cache_get(&db, "avtr_future".into())?.expect("cached avatar should be readable");
    assert_eq!(
        cached.release_status,
        ReleaseStatus::Unknown("future".into())
    );
    assert_eq!(
        serde_json::to_value(cached)?.get("releaseStatus"),
        Some(&json!("future"))
    );
    Ok(())
}

#[test]
fn cache_upsert_many_writes_the_whole_batch() -> Result<(), Error> {
    let dir = TestDir::new("avatar-upsert-many");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;

    let entries = (0..300)
        .map(|index| avatar_entry(&format!("avtr_{index}")))
        .collect::<Vec<_>>();
    let written = avatar_cache_upsert_many(&db, entries)?;

    assert_eq!(written, 300);
    for index in [0, 150, 299] {
        assert!(avatar_cache_get(&db, format!("avtr_{index}"))?.is_some());
    }
    Ok(())
}

#[test]
fn cache_upsert_many_skips_malformed_entries_without_losing_the_batch() -> Result<(), Error> {
    let dir = TestDir::new("avatar-upsert-many-invalid");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;

    let mut invalid = avatar_entry("avtr_placeholder");
    invalid.id = json!("   ");
    let entries = vec![avatar_entry("avtr_a"), invalid, avatar_entry("avtr_b")];

    let written = avatar_cache_upsert_many(&db, entries)?;

    assert_eq!(written, 2);
    assert!(avatar_cache_get(&db, "avtr_a".into())?.is_some());
    assert!(avatar_cache_get(&db, "avtr_b".into())?.is_some());
    Ok(())
}

#[test]
fn cache_upsert_many_overwrites_an_existing_snapshot() -> Result<(), Error> {
    let dir = TestDir::new("avatar-upsert-many-overwrite");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;

    avatar_cache_upsert(&db, avatar_entry("avtr_a"))?;
    let mut updated = avatar_entry("avtr_a");
    updated.name = json!("Renamed Avatar");
    avatar_cache_upsert_many(&db, vec![updated])?;

    assert_eq!(
        avatar_cache_get(&db, "avtr_a".into())?.unwrap().name,
        "Renamed Avatar"
    );
    Ok(())
}

const HOUR_MS: i64 = 3_600_000;

fn write_wear(
    db: &DatabaseService,
    user_id: &str,
    avatar_id: &str,
    started_ms: i64,
    ended_ms: i64,
) -> Result<(), Error> {
    crate::realtime::write_realtime_batch(
        db,
        &crate::ownership::OwnerId::new(user_id),
        &vrcx_0_contracts::realtime::RealtimePersistenceBatch {
            avatar_time_spent_upserts: vec![vrcx_0_contracts::realtime::AvatarTimeSpentUpsert {
                avatar_id: avatar_id.into(),
                created_at: "2026-01-01T00:00:00.000Z".into(),
                time_spent: ended_ms - started_ms,
                started_at_ms: started_ms,
                ended_at_ms: ended_ms,
            }],
            ..Default::default()
        },
    )?;
    Ok(())
}

#[test]
fn windowed_avatar_ranking_clips_sessions_to_the_window() -> Result<(), Error> {
    let dir = TestDir::new("avatar-wear-window");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    let base: i64 = 1_767_225_600_000;

    write_wear(
        &db,
        "usr_a",
        "avtr_inside",
        base + HOUR_MS,
        base + 3 * HOUR_MS,
    )?;
    write_wear(
        &db,
        "usr_a",
        "avtr_straddle",
        base - 5 * HOUR_MS,
        base + HOUR_MS,
    )?;
    write_wear(
        &db,
        "usr_a",
        "avtr_before",
        base - 9 * HOUR_MS,
        base - 6 * HOUR_MS,
    )?;

    let ranked = avatar_usage_ranking_windowed(&db, "usr_a".into(), base, base + 4 * HOUR_MS, 10)?;

    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].avatar_id, "avtr_inside");
    assert_eq!(ranked[0].time_spent, 2 * HOUR_MS);
    assert_eq!(ranked[1].avatar_id, "avtr_straddle");
    assert_eq!(ranked[1].time_spent, HOUR_MS);
    Ok(())
}

#[test]
fn windowed_avatar_ranking_sums_repeat_sessions() -> Result<(), Error> {
    let dir = TestDir::new("avatar-wear-repeat");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    let base: i64 = 1_767_225_600_000;

    write_wear(&db, "usr_a", "avtr_repeat", base, base + HOUR_MS)?;
    write_wear(
        &db,
        "usr_a",
        "avtr_repeat",
        base + 2 * HOUR_MS,
        base + 5 * HOUR_MS,
    )?;
    write_wear(&db, "usr_a", "avtr_single", base, base + 2 * HOUR_MS)?;

    let ranked = avatar_usage_ranking_windowed(&db, "usr_a".into(), base, base + 6 * HOUR_MS, 10)?;

    assert_eq!(ranked[0].avatar_id, "avtr_repeat");
    assert_eq!(ranked[0].time_spent, 4 * HOUR_MS);
    assert_eq!(ranked[1].avatar_id, "avtr_single");
    Ok(())
}
