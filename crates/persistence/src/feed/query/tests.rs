use std::path::PathBuf;

use serde_json::json;
use vrcx_0_core::json::RawJson;

use super::{
    feed_latest_query, feed_row_from_value, feed_rows_query, feed_rows_query_interruptible,
    merge_feed_rows_with_live, FeedCursorInput, FeedFilter, FeedLatestQueryInput,
    FeedLiveEntryInput, FeedLiveRowsMergeContext, FeedQueryMode, FeedReadModelOutput,
    FeedRowsQueryInput,
};
use crate::database::DatabaseService;
use crate::realtime::{write_realtime_batch, RealtimePersistenceBatch};

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

#[derive(Default)]
struct MergeCase {
    rows: Vec<RawJson>,
    current_user_id: String,
    filters: Vec<FeedFilter>,
    search: String,
    date_from: String,
    date_to: String,
    favorites_only: bool,
    favorite_user_ids: Vec<String>,
    scoped_user_ids: Vec<String>,
    excluded_user_ids: Vec<String>,
    live_entries: Vec<FeedLiveEntryInput>,
    min_live_sequence: i64,
    max_rows: i64,
}

fn merge_case(input: MergeCase) -> FeedReadModelOutput {
    let context = FeedLiveRowsMergeContext {
        current_user_id: &input.current_user_id,
        filters: &input.filters,
        search: &input.search,
        date_from: &input.date_from,
        date_to: &input.date_to,
        favorites_only: input.favorites_only,
        favorite_user_ids: &input.favorite_user_ids,
        scoped_user_ids: &input.scoped_user_ids,
        excluded_user_ids: &input.excluded_user_ids,
        max_rows: input.max_rows,
    };
    merge_feed_rows_with_live(
        input
            .rows
            .iter()
            .map(|row| feed_row_from_value(row.as_value()))
            .collect(),
        &input.live_entries,
        input.min_live_sequence,
        context,
    )
}

#[test]
fn live_feed_ignores_friend_relationship_events_without_active_filters() {
    let output = merge_case(MergeCase {
        rows: Vec::new(),
        current_user_id: "usr_self".into(),
        filters: Vec::new(),
        search: String::new(),
        date_from: String::new(),
        date_to: String::new(),
        favorites_only: false,
        favorite_user_ids: Vec::new(),
        scoped_user_ids: Vec::new(),
        excluded_user_ids: Vec::new(),
        live_entries: vec![
            FeedLiveEntryInput {
                sequence: 1,
                entry: RawJson::from(json!({
                    "type": "Friend",
                    "userId": "usr_friend",
                    "displayName": "Friend",
                    "created_at": "2026-05-15T00:00:00Z",
                })),
            },
            FeedLiveEntryInput {
                sequence: 2,
                entry: RawJson::from(json!({
                    "type": "GPS",
                    "userId": "usr_friend",
                    "displayName": "Friend",
                    "location": "wrld_1:instance",
                    "created_at": "2026-05-15T00:00:01Z",
                })),
            },
        ],
        min_live_sequence: 0,
        max_rows: 10,
    });

    assert_eq!(output.max_sequence, 2);
    assert_eq!(output.rows.len(), 1);
    assert_eq!(output.rows[0].r#type.as_deref(), Some("GPS"));
}

#[test]
fn user_scope_drops_live_entries_and_existing_rows_outside_the_scope() {
    let output = merge_case(MergeCase {
        rows: vec![
            RawJson::from(json!({
                "type": "GPS",
                "userId": "usr_scoped",
                "displayName": "Scoped",
                "location": "wrld_1:instance",
                "created_at": "2026-05-15T00:00:00Z",
            })),
            RawJson::from(json!({
                "type": "GPS",
                "userId": "usr_other",
                "displayName": "Other",
                "location": "wrld_2:instance",
                "created_at": "2026-05-15T00:00:01Z",
            })),
        ],
        current_user_id: "usr_self".into(),
        filters: Vec::new(),
        search: String::new(),
        date_from: String::new(),
        date_to: String::new(),
        favorites_only: false,
        favorite_user_ids: Vec::new(),
        scoped_user_ids: vec!["usr_scoped".into()],
        excluded_user_ids: Vec::new(),
        live_entries: vec![
            FeedLiveEntryInput {
                sequence: 1,
                entry: RawJson::from(json!({
                    "type": "GPS",
                    "userId": "usr_other",
                    "displayName": "Other",
                    "location": "wrld_3:instance",
                    "created_at": "2026-05-15T00:00:02Z",
                })),
            },
            FeedLiveEntryInput {
                sequence: 2,
                entry: RawJson::from(json!({
                    "type": "GPS",
                    "userId": "usr_scoped",
                    "displayName": "Scoped",
                    "location": "wrld_4:instance",
                    "created_at": "2026-05-15T00:00:03Z",
                })),
            },
        ],
        min_live_sequence: 0,
        max_rows: 10,
    });

    assert_eq!(output.max_sequence, 2);
    let user_ids = output
        .rows
        .iter()
        .map(|row| row.user_id.as_deref().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(user_ids, vec!["usr_scoped", "usr_scoped"]);
}

#[test]
fn live_feed_rows_keep_avatar_fields_that_only_exist_on_live_entries() {
    let output = merge_case(MergeCase {
        rows: Vec::new(),
        current_user_id: "usr_self".into(),
        filters: Vec::new(),
        search: String::new(),
        date_from: String::new(),
        date_to: String::new(),
        favorites_only: false,
        favorite_user_ids: Vec::new(),
        scoped_user_ids: Vec::new(),
        excluded_user_ids: Vec::new(),
        live_entries: vec![FeedLiveEntryInput {
            sequence: 1,
            entry: RawJson::from(json!({
                "type": "Avatar",
                "userId": "usr_friend",
                "displayName": "Friend",
                "created_at": "2026-05-15T00:00:00Z",
                "avatarName": "Current",
                "previousAvatarName": "Previous",
                "ownerId": "usr_owner",
                "previousOwnerId": "usr_previous_owner",
                "currentAvatarTags": ["content_horror"],
                "previousCurrentAvatarTags": [],
            })),
        }],
        min_live_sequence: 0,
        max_rows: 10,
    });

    assert_eq!(output.rows.len(), 1);
    let row = &output.rows[0];
    assert_eq!(row.previous_avatar_name.as_deref(), Some("Previous"));
    assert_eq!(row.previous_owner_id.as_deref(), Some("usr_previous_owner"));
    assert_eq!(
        row.current_avatar_tags.as_deref(),
        Some(["content_horror".to_string()].as_slice())
    );
    assert_eq!(
        row.previous_current_avatar_tags.as_deref(),
        Some([].as_slice())
    );
    assert_eq!(row.row_id, None);
}

#[test]
fn live_avatar_search_matches_private_and_public_with_dates_and_filters() {
    let live_entries = vec![
        FeedLiveEntryInput {
            sequence: 1,
            entry: RawJson::from(json!({
                "type": "Avatar",
                "userId": "usr_old",
                "ownerId": "usr_old",
                "avatarName": "Old",
                "created_at": "2026-05-01T00:00:00Z",
            })),
        },
        FeedLiveEntryInput {
            sequence: 2,
            entry: RawJson::from(json!({
                "type": "Avatar",
                "userId": "usr_private",
                "ownerId": "usr_private",
                "avatarName": "Owned Avatar",
                "created_at": "2026-05-20T00:00:00Z",
            })),
        },
        FeedLiveEntryInput {
            sequence: 3,
            entry: RawJson::from(json!({
                "type": "Avatar",
                "userId": "usr_public",
                "ownerId": "usr_author",
                "avatarName": "Shared Avatar",
                "created_at": "2026-05-21T00:00:00Z",
            })),
        },
        FeedLiveEntryInput {
            sequence: 4,
            entry: RawJson::from(json!({
                "type": "Avatar",
                "userId": "usr_missing_owner",
                "avatarName": "Missing Owner",
                "created_at": "2026-05-21T00:00:00Z",
            })),
        },
        FeedLiveEntryInput {
            sequence: 5,
            entry: RawJson::from(json!({
                "type": "GPS",
                "userId": "usr_gps",
                "ownerId": "usr_other",
                "created_at": "2026-05-20T00:00:00Z",
            })),
        },
    ];

    let private_output = merge_case(MergeCase {
        filters: vec![FeedFilter::Avatar],
        search: "private".into(),
        date_from: "2026-05-10T00:00:00Z".into(),
        date_to: "2026-05-20T00:00:00Z".into(),
        live_entries: live_entries.clone(),
        max_rows: 10,
        ..MergeCase::default()
    });
    assert_eq!(private_output.rows.len(), 1);
    assert_eq!(
        private_output.rows[0].user_id.as_deref(),
        Some("usr_private")
    );

    let public_output = merge_case(MergeCase {
        filters: vec![FeedFilter::Avatar],
        search: "public".into(),
        date_from: "2026-05-21T00:00:00Z".into(),
        date_to: "2026-05-21T00:00:00Z".into(),
        live_entries,
        max_rows: 10,
        ..MergeCase::default()
    });
    assert_eq!(public_output.rows.len(), 1);
    assert_eq!(public_output.rows[0].user_id.as_deref(), Some("usr_public"));
}

#[test]
fn merged_rows_normalize_snake_case_live_entry_field_names() {
    let output = merge_case(MergeCase {
        rows: Vec::new(),
        current_user_id: "usr_self".into(),
        filters: Vec::new(),
        search: String::new(),
        date_from: String::new(),
        date_to: String::new(),
        favorites_only: false,
        favorite_user_ids: Vec::new(),
        scoped_user_ids: Vec::new(),
        excluded_user_ids: Vec::new(),
        live_entries: vec![FeedLiveEntryInput {
            sequence: 1,
            entry: RawJson::from(json!({
                "type": "GPS",
                "user_id": "usr_friend",
                "display_name": "Friend",
                "createdAt": "2026-05-15T00:00:00Z",
                "location": "wrld_1:instance",
                "world_name": "World",
                "time": "1500",
            })),
        }],
        min_live_sequence: 0,
        max_rows: 10,
    });

    assert_eq!(output.rows.len(), 1);
    let row = &output.rows[0];
    assert_eq!(row.user_id.as_deref(), Some("usr_friend"));
    assert_eq!(row.display_name.as_deref(), Some("Friend"));
    assert_eq!(row.created_at.as_deref(), Some("2026-05-15T00:00:00Z"));
    assert_eq!(row.world_name.as_deref(), Some("World"));
    assert_eq!(row.time, Some(1500));
}

#[test]
fn persisted_rows_with_matching_content_identity_remain_distinct() {
    let output = merge_case(MergeCase {
        rows: vec![
            RawJson::from(json!({
                "rowId": 1,
                "sourceRank": 40,
                "type": "Status",
                "userId": "usr_friend",
                "created_at": "2026-05-15T00:00:00Z",
                "status": "active"
            })),
            RawJson::from(json!({
                "rowId": 2,
                "sourceRank": 40,
                "type": "Status",
                "userId": "usr_friend",
                "created_at": "2026-05-15T00:00:00Z",
                "status": "join me"
            })),
        ],
        current_user_id: "usr_self".into(),
        max_rows: 10,
        ..MergeCase::default()
    });

    assert_eq!(output.rows.len(), 2);
    assert_eq!(output.rows[0].row_id, Some(1));
    assert_eq!(output.rows[1].row_id, Some(2));
}

#[test]
fn live_row_replaces_the_same_persisted_feed_entry() {
    let output = merge_case(MergeCase {
        rows: vec![RawJson::from(json!({
            "rowId": 1,
            "sourceRank": 60,
            "type": "GPS",
            "userId": "usr_friend",
            "created_at": "2026-05-15T00:00:00Z",
            "location": "wrld_1:instance"
        }))],
        current_user_id: "usr_self".into(),
        live_entries: vec![FeedLiveEntryInput {
            sequence: 1,
            entry: RawJson::from(json!({
                "type": "GPS",
                "userId": "usr_friend",
                "created_at": "2026-05-15T00:00:00Z",
                "location": "wrld_1:instance"
            })),
        }],
        max_rows: 10,
        ..MergeCase::default()
    });

    assert_eq!(output.rows.len(), 1);
    assert_eq!(output.rows[0].row_id, None);
    assert_eq!(output.max_sequence, 1);
}

#[test]
fn latest_query_keeps_the_persisted_cursor_when_live_rows_fill_the_result(
) -> Result<(), crate::Error> {
    let dir = TestDir::new("feed-latest-persisted-cursor");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    write_realtime_batch(
        &db,
        "usr_self",
        &RealtimePersistenceBatch {
            feed_entries: vec![json!({
                "created_at": "2026-05-15T00:00:00Z",
                "type": "GPS",
                "userId": "usr_persisted",
                "displayName": "Persisted",
                "location": "wrld_1:persisted",
                "worldName": "Persisted World",
                "previousLocation": "",
                "time": 0,
                "groupName": ""
            })],
            ..RealtimePersistenceBatch::default()
        },
    )?;

    let output = feed_latest_query(
        &db,
        FeedLatestQueryInput {
            user_id: "usr_self".into(),
            filters: vec![FeedFilter::Gps],
            favorite_user_ids: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            favorites_only: false,
            max_rows: 1,
        },
        vec![FeedLiveEntryInput {
            sequence: 1,
            entry: RawJson::from(json!({
                "type": "GPS",
                "userId": "usr_live",
                "created_at": "2026-05-15T00:01:00Z",
                "location": "wrld_1:live"
            })),
        }],
        1,
        true,
    )?;

    assert_eq!(output.rows.len(), 1);
    assert_eq!(output.rows[0].user_id.as_deref(), Some("usr_live"));
    assert_eq!(output.rows[0].row_id, None);
    assert!(output.persisted_has_more);
    assert!(output.persisted_cursor.is_some());
    Ok(())
}

#[test]
fn lookup_feed_pagination_uses_the_same_date_order_as_its_cursor() -> Result<(), crate::Error> {
    let dir = TestDir::new("feed-lookup-rowid");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;

    write_realtime_batch(
        &db,
        "usr_self",
        &RealtimePersistenceBatch {
            feed_entries: vec![
                json!({
                    "created_at": "2026-05-15T00:10:00Z",
                    "type": "GPS",
                    "userId": "usr_newer_created",
                    "displayName": "newer-created",
                    "location": "wrld_1:newer",
                    "worldName": "Newer Created",
                    "previousLocation": "",
                    "time": 0,
                    "groupName": ""
                }),
                json!({
                    "created_at": "2026-05-15T00:00:00Z",
                    "type": "GPS",
                    "userId": "usr_later_inserted",
                    "displayName": "later-inserted",
                    "location": "wrld_1:later",
                    "worldName": "Later Inserted",
                    "previousLocation": "",
                    "time": 0,
                    "groupName": ""
                }),
            ],
            ..RealtimePersistenceBatch::default()
        },
    )?;

    let first_page = feed_rows_query(
        &db,
        FeedRowsQueryInput {
            user_id: "usr_self".into(),
            mode: FeedQueryMode::Lookup,
            search: String::new(),
            filters: vec![FeedFilter::Gps],
            vip_list: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            max_entries: 1,
            date_from: String::new(),
            date_to: String::new(),
            cursor: None,
        },
    )?;

    assert_eq!(first_page.len(), 1);
    assert_eq!(first_page[0].display_name.as_deref(), Some("newer-created"));

    let second_page = feed_rows_query(
        &db,
        FeedRowsQueryInput {
            user_id: "usr_self".into(),
            mode: FeedQueryMode::Lookup,
            search: String::new(),
            filters: vec![FeedFilter::Gps],
            vip_list: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            max_entries: 1,
            date_from: String::new(),
            date_to: String::new(),
            cursor: Some(FeedCursorInput {
                created_at: first_page[0].created_at.clone().unwrap(),
                source_rank: first_page[0].source_rank.unwrap(),
                row_id: first_page[0].row_id.unwrap(),
            }),
        },
    )?;

    assert_eq!(second_page.len(), 1);
    assert_eq!(
        second_page[0].display_name.as_deref(),
        Some("later-inserted")
    );
    Ok(())
}

#[test]
fn world_id_search_honors_the_date_window() -> Result<(), crate::Error> {
    let dir = TestDir::new("feed-search-world-date-window");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    write_realtime_batch(
        &db,
        "usr_self",
        &RealtimePersistenceBatch {
            feed_entries: vec![
                json!({
                    "created_at": "2026-05-01T00:00:00Z",
                    "type": "GPS",
                    "userId": "usr_old",
                    "displayName": "Old",
                    "location": "wrld_target:old",
                    "worldName": "Target",
                }),
                json!({
                    "created_at": "2026-05-20T00:00:00Z",
                    "type": "GPS",
                    "userId": "usr_new",
                    "displayName": "New",
                    "location": "wrld_target:new",
                    "worldName": "Target",
                }),
            ],
            ..RealtimePersistenceBatch::default()
        },
    )?;

    let rows = feed_rows_query_interruptible(
        &db,
        FeedRowsQueryInput {
            user_id: "usr_self".into(),
            mode: FeedQueryMode::Search,
            search: "wrld_target".into(),
            filters: vec![FeedFilter::Gps],
            vip_list: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            max_entries: 10,
            date_from: "2026-05-10T00:00:00Z".into(),
            date_to: String::new(),
            cursor: None,
        },
        || false,
    )?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].user_id.as_deref(), Some("usr_new"));
    Ok(())
}

#[test]
fn private_avatar_search_applies_dates_to_every_match_branch() -> Result<(), crate::Error> {
    let dir = TestDir::new("feed-search-private-avatar-date-window");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    write_realtime_batch(
        &db,
        "usr_self",
        &RealtimePersistenceBatch {
            feed_entries: vec![
                json!({
                    "created_at": "2026-05-01T00:00:00Z",
                    "type": "Avatar",
                    "userId": "usr_old",
                    "displayName": "Private Collector",
                    "ownerId": "usr_old",
                    "avatarName": "Old Avatar",
                }),
                json!({
                    "created_at": "2026-05-20T00:00:00Z",
                    "type": "Avatar",
                    "userId": "usr_new",
                    "displayName": "New",
                    "ownerId": "usr_new",
                    "avatarName": "New Avatar",
                }),
            ],
            ..RealtimePersistenceBatch::default()
        },
    )?;

    let rows = feed_rows_query_interruptible(
        &db,
        FeedRowsQueryInput {
            user_id: "usr_self".into(),
            mode: FeedQueryMode::Search,
            search: "private".into(),
            filters: vec![FeedFilter::Avatar],
            vip_list: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            max_entries: 10,
            date_from: "2026-05-10T00:00:00Z".into(),
            date_to: String::new(),
            cursor: None,
        },
        || false,
    )?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].user_id.as_deref(), Some("usr_new"));
    Ok(())
}

#[test]
fn date_window_preserves_millisecond_boundaries() -> Result<(), crate::Error> {
    let dir = TestDir::new("feed-millisecond-date-window");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    write_realtime_batch(
        &db,
        "usr_self",
        &RealtimePersistenceBatch {
            feed_entries: vec![
                json!({
                    "created_at": "2026-05-20T00:00:00.000Z",
                    "type": "Status",
                    "userId": "usr_milliseconds",
                    "displayName": "Milliseconds",
                    "status": "active",
                }),
                json!({
                    "created_at": "2026-05-20T00:00:00Z",
                    "type": "Status",
                    "userId": "usr_seconds",
                    "displayName": "Seconds",
                    "status": "active",
                }),
                json!({
                    "created_at": "2026-05-20T00:00:00+00:00",
                    "type": "Status",
                    "userId": "usr_offset",
                    "displayName": "Offset",
                    "status": "active",
                }),
                json!({
                    "created_at": "2026-05-20T00:00:00.500Z",
                    "type": "Status",
                    "userId": "usr_later",
                    "displayName": "Later",
                    "status": "active",
                }),
            ],
            ..RealtimePersistenceBatch::default()
        },
    )?;

    let rows = feed_rows_query_interruptible(
        &db,
        FeedRowsQueryInput {
            user_id: "usr_self".into(),
            mode: FeedQueryMode::Lookup,
            search: String::new(),
            filters: vec![FeedFilter::Status],
            vip_list: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            max_entries: 10,
            date_from: "2026-05-20T00:00:00.000Z".into(),
            date_to: "2026-05-20T00:00:00.000Z".into(),
            cursor: None,
        },
        || false,
    )?;

    assert_eq!(rows.len(), 3);
    assert!(rows
        .iter()
        .all(|row| row.user_id.as_deref() != Some("usr_later")));

    let newest = feed_rows_query_interruptible(
        &db,
        FeedRowsQueryInput {
            user_id: "usr_self".into(),
            mode: FeedQueryMode::Lookup,
            search: String::new(),
            filters: vec![FeedFilter::Status],
            vip_list: Vec::new(),
            scoped_user_ids: Vec::new(),
            excluded_user_ids: Vec::new(),
            max_entries: 1,
            date_from: String::new(),
            date_to: String::new(),
            cursor: None,
        },
        || false,
    )?;
    assert_eq!(newest[0].user_id.as_deref(), Some("usr_later"));
    Ok(())
}

#[test]
fn search_matches_previous_values_and_escapes_like_wildcards() -> Result<(), crate::Error> {
    let dir = TestDir::new("feed-search-previous-literal");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?;
    write_realtime_batch(
        &db,
        "usr_self",
        &RealtimePersistenceBatch {
            feed_entries: vec![
                json!({
                    "created_at": "2026-05-20T00:00:00.000Z",
                    "type": "Bio",
                    "userId": "usr_previous",
                    "displayName": "Previous",
                    "bio": "new value",
                    "previousBio": "removed needle",
                }),
                json!({
                    "created_at": "2026-05-19T00:00:00.000Z",
                    "type": "Bio",
                    "userId": "usr_percent",
                    "displayName": "Percent",
                    "bio": "100% literal",
                    "previousBio": "old",
                }),
                json!({
                    "created_at": "2026-05-18T00:00:00.000Z",
                    "type": "Bio",
                    "userId": "usr_other",
                    "displayName": "Other",
                    "bio": "ordinary text",
                    "previousBio": "old",
                }),
            ],
            ..RealtimePersistenceBatch::default()
        },
    )?;

    let search = |text: &str| {
        feed_rows_query(
            &db,
            FeedRowsQueryInput {
                user_id: "usr_self".into(),
                mode: FeedQueryMode::Search,
                search: text.into(),
                filters: vec![FeedFilter::Bio],
                vip_list: Vec::new(),
                scoped_user_ids: Vec::new(),
                excluded_user_ids: Vec::new(),
                max_entries: 10,
                date_from: String::new(),
                date_to: String::new(),
                cursor: None,
            },
        )
    };

    let previous = search("removed needle")?;
    assert_eq!(previous.len(), 1);
    assert_eq!(previous[0].user_id.as_deref(), Some("usr_previous"));

    let literal_percent = search("%")?;
    assert_eq!(literal_percent.len(), 1);
    assert_eq!(literal_percent[0].user_id.as_deref(), Some("usr_percent"));
    Ok(())
}
