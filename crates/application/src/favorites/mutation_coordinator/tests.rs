use std::{path::PathBuf, sync::Arc};

use serde_json::json;
use vrcx_0_persistence::{
    favorites,
    social_aggregates::{FavoriteAction, FavoriteLocalInput},
    storage::StorageService,
};

use super::*;
use crate::{
    FavoriteBulkRemoveItem, FavoriteBulkRemoveSource, FavoriteTransferInput, FavoriteTransferItem,
    FavoriteTransferLocation, FavoriteTransferMode, FavoriteTransferSource, FavoriteTransferTarget,
};

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vrcx-0-favorite-mutations-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Harness {
    _dir: TestDir,
    coordinator: FavoriteMutationCoordinator,
    db: Arc<DatabaseService>,
    event_bus: RuntimeEventBus,
}

fn harness(name: &str) -> Harness {
    let dir = TestDir::new(name);
    let db = Arc::new(DatabaseService::new(&dir.0.join("VRCX-0.sqlite3")).unwrap());
    let storage = StorageService::new(&dir.0.join("storage.json")).unwrap();
    let web = Arc::new(
        WebClient::new(
            &storage,
            db.as_ref(),
            "wss://pipeline.vrchat.cloud".into(),
            env!("CARGO_PKG_VERSION"),
        )
        .unwrap(),
    );
    let auth_scope = RuntimeAuthScope::new();
    auth_scope.set("usr_self", "https://api.vrchat.cloud/api/1");
    let event_bus = RuntimeEventBus::new();
    let coordinator = FavoriteMutationCoordinator::new(
        Arc::clone(&db),
        web,
        RuntimeDiagnostics::new(),
        RuntimeSyncEngine::new(),
        event_bus.clone(),
        auth_scope,
        Arc::new(RemoteMutationGate::default()),
    );
    Harness {
        _dir: dir,
        coordinator,
        db,
        event_bus,
    }
}

fn assert_single_local_invalidation(event_bus: &RuntimeEventBus) {
    let events = event_bus.take_events_for_test();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "favoritesChanged");
    assert_eq!(
        events[0].payload,
        json!({
            "ownerUserId": "usr_self",
            "endpoint": "https://api.vrchat.cloud/api/1",
            "kind": "friend",
            "local": true,
            "remote": false,
            "changes": [],
            "requiresRefresh": true
        })
    );
}

#[test]
fn local_mutation_persists_and_emits_one_exact_delta() {
    let harness = harness("local-delta");

    let affected = harness
        .coordinator
        .add_local(
            FavoriteEntityKind::Friend,
            "usr_friend".into(),
            "Close".into(),
        )
        .unwrap();

    assert_eq!(affected, 1);
    assert_eq!(
        favorites::favorite_list(
            harness.db.as_ref(),
            Some(&OwnerId::new("usr_self")),
            FavoriteEntityKind::Friend,
        )
        .unwrap()
        .len(),
        1
    );
    let events = harness.event_bus.take_events_for_test();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "favoritesChanged");
    assert_eq!(
        events[0].payload,
        json!({
            "ownerUserId": "usr_self",
            "endpoint": "https://api.vrchat.cloud/api/1",
            "kind": "friend",
            "local": true,
            "remote": false,
            "changes": [{
                "type": "localAdded",
                "kind": "friend",
                "entityId": "usr_friend",
                "groupName": "Close"
            }],
            "requiresRefresh": false
        })
    );
}

#[test]
fn tool_dry_run_does_not_persist_or_emit() {
    let harness = harness("tool-dry-run");

    let output = harness
        .coordinator
        .mutate_local(
            "MCP local favorite mutation",
            FavoriteLocalInput {
                kind: FavoriteEntityKind::Friend,
                entity_id: "usr_friend".into(),
                group: "Close".into(),
                action: FavoriteAction::Add,
                dry_run: true,
            },
        )
        .unwrap();

    assert_eq!(output.affected_rows, 0);
    assert!(favorites::favorite_list(
        harness.db.as_ref(),
        Some(&OwnerId::new("usr_self")),
        FavoriteEntityKind::Friend,
    )
    .unwrap()
    .is_empty());
    assert!(harness.event_bus.take_events_for_test().is_empty());
}

#[test]
fn tool_write_persists_and_emits_one_invalidation() {
    let harness = harness("tool-write");

    let output = harness
        .coordinator
        .mutate_local(
            "MCP local favorite mutation",
            FavoriteLocalInput {
                kind: FavoriteEntityKind::Friend,
                entity_id: "usr_friend".into(),
                group: "Close".into(),
                action: FavoriteAction::Add,
                dry_run: false,
            },
        )
        .unwrap();

    assert_eq!(output.affected_rows, 1);
    assert_single_local_invalidation(&harness.event_bus);
}

#[tokio::test]
async fn local_transfer_emits_once_with_exact_changed_sides() {
    let harness = harness("local-transfer");
    favorites::favorite_add(
        harness.db.as_ref(),
        Some(&OwnerId::new("usr_self")),
        FavoriteEntityKind::Friend,
        "usr_friend".into(),
        "Source".into(),
    )
    .unwrap();

    let output = harness
        .coordinator
        .transfer_selection(FavoriteTransferSelectionInput {
            batches: vec![FavoriteTransferInput {
                kind: FavoriteEntityKind::Friend,
                mode: FavoriteTransferMode::Move,
                source: FavoriteTransferSource {
                    location: FavoriteTransferLocation::Local,
                    group: "Source".into(),
                },
                target: FavoriteTransferTarget {
                    location: FavoriteTransferLocation::Local,
                    group: "Target".into(),
                    favorite_type: None,
                },
                items: vec![FavoriteTransferItem {
                    key: "local:Source:usr_friend".into(),
                    entity_id: "usr_friend".into(),
                    entity: None,
                }],
            }],
        })
        .await
        .unwrap();

    assert_eq!(output.succeeded, 1);
    assert_eq!(output.failed, 0);
    assert!(output.local_changed);
    assert!(!output.remote_changed);
    let rows = favorites::favorite_list(
        harness.db.as_ref(),
        Some(&OwnerId::new("usr_self")),
        FavoriteEntityKind::Friend,
    )
    .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].group_name, "Target");
    assert_single_local_invalidation(&harness.event_bus);
}

#[tokio::test]
async fn local_bulk_remove_emits_once_with_exact_changed_sides() {
    let harness = harness("local-bulk-remove");
    favorites::favorite_add(
        harness.db.as_ref(),
        Some(&OwnerId::new("usr_self")),
        FavoriteEntityKind::Friend,
        "usr_friend".into(),
        "Close".into(),
    )
    .unwrap();

    let output = harness
        .coordinator
        .remove_selection(FavoriteBulkRemoveInput {
            kind: FavoriteEntityKind::Friend,
            items: vec![FavoriteBulkRemoveItem {
                key: "local:Close:usr_friend".into(),
                source: FavoriteBulkRemoveSource::Local,
                entity_id: "usr_friend".into(),
                group_name: "Close".into(),
            }],
        })
        .await
        .unwrap();

    assert_eq!(output.succeeded, 1);
    assert_eq!(output.failed, 0);
    assert!(output.local_changed);
    assert!(!output.remote_changed);
    assert_single_local_invalidation(&harness.event_bus);
}

#[test]
fn import_completion_emits_only_for_successful_import_writes() {
    let harness = harness("import-completion");
    let scope = RuntimeAuthScopeSnapshot {
        current_user_id: "usr_self".into(),
        endpoint: "https://api.vrchat.cloud/api/1".into(),
        generation: 1,
        active: true,
    };
    let mut status = FavoriteImportStatus {
        operation: FavoriteImportOperation::Hydrate,
        kind: FavoriteEntityKind::Friend,
        succeeded: 1,
        ..FavoriteImportStatus::default()
    };

    harness
        .coordinator
        .complete_import(&scope, &status, Some(FavoriteImportLocation::Local));
    assert!(harness.event_bus.take_events_for_test().is_empty());

    status.operation = FavoriteImportOperation::Import;
    harness
        .coordinator
        .complete_import(&scope, &status, Some(FavoriteImportLocation::Remote));
    let events = harness.event_bus.take_events_for_test();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "favoritesChanged");
    assert_eq!(events[0].payload["local"], false);
    assert_eq!(events[0].payload["remote"], true);
    assert_eq!(events[0].payload["requiresRefresh"], true);
}
