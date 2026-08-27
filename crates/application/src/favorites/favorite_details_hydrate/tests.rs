use super::*;
use serde_json::json;
use vrcx_0_application_core::MemoryWorldCachePort;

fn complete(release_status: &str) -> Value {
    json!({
        "id": "avtr_1",
        "name": "Entity",
        "releaseStatus": release_status,
        "thumbnailImageUrl": "https://example.test/thumb.png",
    })
}

#[test]
fn avatar_decision_upserts_public_complete_snapshots() {
    assert_eq!(
        cache_write_decision(FavoriteCacheKind::Avatar, &complete("public")),
        CacheWriteDecision::Upsert
    );
}

#[test]
fn avatar_decision_inserts_non_public_complete_snapshots_only_when_missing() {
    for status in ["private", "hidden", ""] {
        assert_eq!(
            cache_write_decision(FavoriteCacheKind::Avatar, &complete(status)),
            CacheWriteDecision::InsertIfMissing
        );
    }
}

#[test]
fn avatar_decision_skips_incomplete_snapshots() {
    assert_eq!(
        cache_write_decision(
            FavoriteCacheKind::Avatar,
            &json!({ "id": "avtr_1", "releaseStatus": "public" }),
        ),
        CacheWriteDecision::Skip
    );
    assert_eq!(
        cache_write_decision(
            FavoriteCacheKind::Avatar,
            &json!({
                "id": "avtr_1",
                "name": "Broken Avatar",
                "releaseStatus": "public",
            })
        ),
        CacheWriteDecision::Skip
    );
}

#[test]
fn avatar_decision_normalizes_release_status_case_and_whitespace() {
    let mut entity = complete("  Public  ");
    assert_eq!(
        cache_write_decision(FavoriteCacheKind::Avatar, &entity),
        CacheWriteDecision::Upsert
    );
    entity["imageUrl"] = json!("https://example.test/image.png");
    entity["thumbnailImageUrl"] = json!("   ");
    assert_eq!(
        cache_write_decision(FavoriteCacheKind::Avatar, &entity),
        CacheWriteDecision::Upsert
    );
}

#[test]
fn world_decision_upserts_public_complete_snapshots() {
    assert_eq!(
        cache_write_decision(FavoriteCacheKind::World, &complete("public")),
        CacheWriteDecision::Upsert
    );
}

#[test]
fn world_decision_inserts_private_complete_snapshots_only_when_missing() {
    assert_eq!(
        cache_write_decision(FavoriteCacheKind::World, &complete("private")),
        CacheWriteDecision::InsertIfMissing
    );
}

#[test]
fn world_decision_skips_other_release_statuses_unlike_avatars() {
    for status in ["hidden", "labs", ""] {
        assert_eq!(
            cache_write_decision(FavoriteCacheKind::World, &complete(status)),
            CacheWriteDecision::Skip
        );
    }
}

#[test]
fn world_decision_skips_incomplete_snapshots() {
    assert_eq!(
        cache_write_decision(
            FavoriteCacheKind::World,
            &json!({
                "id": "wrld_1",
                "name": "World",
                "releaseStatus": "public",
            })
        ),
        CacheWriteDecision::Skip
    );
}

#[test]
fn filter_keeps_only_requested_favorite_ids() {
    let entities = vec![
        json!({ "id": "wrld_1", "name": "One" }),
        json!({ "id": " wrld_2 ", "name": "Two" }),
        json!({ "id": "wrld_3", "name": "Three" }),
        json!({ "name": "No id" }),
    ];

    let details = filter_details_by_id(entities, &["wrld_2".into(), " wrld_3 ".into()]);

    assert_eq!(details.len(), 2);
    assert!(details.contains_key("wrld_2"));
    assert!(details.contains_key("wrld_3"));
}

#[test]
fn filter_keeps_everything_when_favorite_ids_are_empty() {
    let entities = vec![
        json!({ "id": "wrld_1" }),
        json!({ "id": "wrld_2" }),
        json!({ "name": "No id" }),
    ];

    let details = filter_details_by_id(entities, &[]);

    assert_eq!(details.len(), 2);
}

#[test]
fn merge_avatar_rows_deduplicates_across_tag_pages() {
    let mut seen_ids = HashSet::new();
    let mut entities = Vec::new();

    merge_avatar_rows(
        vec![
            json!({ "id": "avtr_1", "name": "First" }),
            json!({ "id": "avtr_2" }),
        ],
        &mut seen_ids,
        &mut entities,
    );
    merge_avatar_rows(
        vec![
            json!({ "id": " avtr_1 ", "name": "Duplicate" }),
            json!({ "id": "" }),
            json!({ "id": "avtr_3" }),
        ],
        &mut seen_ids,
        &mut entities,
    );

    let ids = entities.iter().map(entity_id).collect::<Vec<_>>();
    assert_eq!(ids, vec!["avtr_1", "avtr_2", "avtr_3"]);
    assert_eq!(entities[0]["name"], json!("First"));
}

#[test]
fn normalize_avatar_tags_deduplicates_and_falls_back_to_single_untagged_round() {
    assert_eq!(
        normalize_avatar_tags(&[" one ".into(), "one".into(), "two".into(), "  ".into()]),
        vec!["one".to_string(), "two".to_string()]
    );
    assert_eq!(normalize_avatar_tags(&[]), vec![String::new()]);
    assert_eq!(normalize_avatar_tags(&["  ".into()]), vec![String::new()]);
}

#[test]
fn missing_world_ids_returns_favorites_without_displayable_details() {
    let details_by_id = HashMap::from([
        ("wrld_named".to_string(), json!({ "name": "Named" })),
        ("wrld_tagged".to_string(), json!({ "tags": ["tag"] })),
        ("wrld_blank".to_string(), json!({ "name": "   " })),
        ("wrld_empty".to_string(), json!({})),
    ]);

    let missing = missing_world_ids(
        &[
            " wrld_named ".to_string(),
            "wrld_tagged".to_string(),
            "wrld_blank".to_string(),
            "wrld_empty".to_string(),
            "wrld_absent".to_string(),
            "wrld_absent".to_string(),
            "  ".to_string(),
        ],
        &details_by_id,
    );

    assert_eq!(missing, vec!["wrld_blank", "wrld_empty", "wrld_absent"]);
}

#[test]
fn world_probe_marks_http_404_as_deleted() {
    assert_eq!(
        classify_world_probe(404, json!({ "error": { "message": "not found" } })),
        WorldProbeOutcome::Deleted
    );
}

#[test]
fn world_probe_failures_do_not_produce_availability() {
    assert_eq!(
        classify_world_probe(500, json!({ "message": "boom" })),
        WorldProbeOutcome::Failed
    );
    assert_eq!(
        classify_world_probe(200, json!({ "error": { "message": "soft error" } })),
        WorldProbeOutcome::Failed
    );
}

#[test]
fn world_probe_classifies_release_status_into_public_or_private() {
    let world = json!({ "id": "wrld_1", "name": "World", "releaseStatus": "Public" });
    assert_eq!(
        classify_world_probe(200, world.clone()),
        WorldProbeOutcome::Available(world, "public".to_string())
    );

    for status in ["private", "hidden", ""] {
        let world = json!({ "id": "wrld_1", "releaseStatus": status });
        assert_eq!(
            classify_world_probe(200, world.clone()),
            WorldProbeOutcome::Available(world, "private".to_string())
        );
    }
}

#[test]
fn world_details_hydrate_uses_cache_and_projects_only_requested_card_fields() {
    let world_cache = WorldCache::new(MemoryWorldCachePort::default());
    let details_by_id = HashMap::from([
        (
            "wrld_requested".to_string(),
            json!({
                "id": "wrld_requested",
                "name": "Requested World",
                "authorId": "usr_author",
                "authorName": "Author",
                "description": "Description",
                "imageUrl": "https://example.test/world.png",
                "releaseStatus": "public",
                "thumbnailImageUrl": "https://example.test/thumb.png",
                "tags": ["author_tag_example"],
                "occupants": 7,
                "unityPackages": [{ "assetUrl": "https://example.test/large.bundle" }],
                "instances": [["123", 4]]
            }),
        ),
        (
            "wrld_unrequested".to_string(),
            json!({
                "id": "wrld_unrequested",
                "name": "Unrequested World",
                "imageUrl": "https://example.test/other.png",
                "releaseStatus": "public"
            }),
        ),
    ]);

    let (details, cached_count) = hydrate_world_details(
        &world_cache,
        details_by_id,
        &[" wrld_requested ".to_string()],
    );

    assert_eq!(cached_count, 2);
    assert_eq!(details.len(), 1);
    let requested = details.get("wrld_requested").unwrap();
    assert_eq!(requested["name"], "Requested World");
    assert_eq!(requested["tags"], json!(["author_tag_example"]));
    assert_eq!(requested["occupants"], 7);
    assert!(requested.get("unityPackages").is_none());
    assert!(requested.get("instances").is_none());
    assert_eq!(
        world_cache
            .get_summary("wrld_unrequested")
            .unwrap()
            .unwrap()
            .name,
        "Unrequested World"
    );
}

#[test]
fn cache_entry_maps_snake_and_camel_timestamps_with_version_fallback() {
    let entity = json!({
        "id": "avtr_1",
        "authorId": " usr_author ",
        "authorName": "Author",
        "createdAt": "2026-06-01T00:00:00.000Z",
        "updated_at": "2026-06-02T00:00:00.000Z",
        "description": "Desc",
        "imageUrl": "https://example.test/image.png",
        "name": "Entity",
        "releaseStatus": "public",
        "thumbnailImageUrl": "https://example.test/thumb.png",
        "version": 7,
    });

    let entry = cache_entry_from_entity(&entity, "avtr_fallback");

    assert_eq!(entry.id, json!("avtr_1"));
    assert_eq!(entry.author_id, json!("usr_author"));
    assert_eq!(entry.created_at, json!("2026-06-01T00:00:00.000Z"));
    assert_eq!(entry.updated_at, json!("2026-06-02T00:00:00.000Z"));
    assert_eq!(entry.version, json!(7));

    let sparse = json!({ "name": "Fallback", "version": "not-a-number" });
    let entry = cache_entry_from_entity(&sparse, " avtr_fallback ");
    assert_eq!(entry.id, json!("avtr_fallback"));
    assert_eq!(entry.version, json!(0));
}
