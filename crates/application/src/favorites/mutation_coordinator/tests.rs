use std::sync::Arc;

use serde_json::json;
use vrcx_0_application_core::NoopWebClientPort;
use vrcx_0_contracts::social_aggregates::{FavoriteAction, FavoriteLocalInput};

use super::*;
use crate::favorites::test_support::TestFavoriteStore;
use crate::favorites::{
    FavoriteBulkRemoveItem, FavoriteBulkRemoveSource, FavoriteRemoteRequests, FavoriteStore,
    FavoriteTransferInput, FavoriteTransferItem, FavoriteTransferLocation, FavoriteTransferMode,
    FavoriteTransferSource, FavoriteTransferTarget,
};

struct TestFavoriteRemoteRequests;

impl FavoriteRemoteRequests for TestFavoriteRemoteRequests {
    fn list(
        &self,
        _endpoint: String,
        _n: i32,
        _offset: i32,
    ) -> vrcx_0_application_core::vrchat_api::VrchatApiRequest {
        Default::default()
    }

    fn limits(&self, _endpoint: String) -> vrcx_0_application_core::vrchat_api::VrchatApiRequest {
        Default::default()
    }

    fn favorite_worlds(
        &self,
        _endpoint: String,
        _n: i32,
        _offset: i32,
        _owner_id: String,
        _user_id: String,
        _tag: String,
    ) -> vrcx_0_application_core::vrchat_api::VrchatApiRequest {
        Default::default()
    }

    fn favorite_avatars(
        &self,
        _endpoint: String,
        _n: i32,
        _offset: i32,
        _tag: String,
    ) -> vrcx_0_application_core::vrchat_api::VrchatApiRequest {
        Default::default()
    }

    fn world(
        &self,
        _endpoint: String,
        world_id: String,
    ) -> Result<(
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((world_id, Default::default()))
    }

    fn avatar(
        &self,
        _endpoint: String,
        avatar_id: String,
    ) -> Result<(
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((avatar_id, Default::default()))
    }

    fn user(
        &self,
        _endpoint: String,
        user_id: String,
    ) -> Result<(
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((user_id, Default::default()))
    }

    fn add(
        &self,
        _endpoint: String,
        input: FavoriteRemoteAddInput,
    ) -> Result<(
        String,
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((
            input.kind.as_str().to_string(),
            input.entity_id,
            Default::default(),
        ))
    }

    fn delete(
        &self,
        _endpoint: String,
        object_id: String,
    ) -> Result<(
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((object_id, Default::default()))
    }

    fn save_group(
        &self,
        _endpoint: String,
        _current_user_id: String,
        input: FavoriteRemoteGroupSaveInput,
    ) -> Result<(
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((input.group, Default::default()))
    }

    fn clear_group(
        &self,
        _endpoint: String,
        _current_user_id: String,
        input: FavoriteRemoteGroupClearInput,
    ) -> Result<(
        String,
        vrcx_0_application_core::vrchat_api::VrchatApiRequest,
    )> {
        Ok((input.group, Default::default()))
    }
}

struct Harness {
    coordinator: FavoriteMutationCoordinator,
    store: Arc<TestFavoriteStore>,
    event_bus: RuntimeEventBus,
}

fn harness(_name: &str) -> Harness {
    let store = Arc::new(TestFavoriteStore::default());
    let web = Arc::new(WebClient::new(NoopWebClientPort));
    let auth_scope = RuntimeAuthScope::new();
    auth_scope.set("usr_self", "https://api.vrchat.cloud/api/1");
    let event_bus = RuntimeEventBus::new();
    let coordinator = FavoriteMutationCoordinator::new(
        Arc::clone(&store) as Arc<dyn FavoriteStore>,
        Arc::new(TestFavoriteRemoteRequests),
        FavoriteMutationRuntimeDeps::new(
            web,
            RuntimeDiagnostics::new(),
            RuntimeSyncEngine::new(),
            event_bus.clone(),
            auth_scope,
            Arc::new(RemoteMutationGate::default()),
        ),
    );
    Harness {
        coordinator,
        store,
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
        .into()
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
        harness
            .store
            .list(Some(&OwnerId::new("usr_self")), FavoriteEntityKind::Friend,)
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
        .into()
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
    assert!(harness
        .store
        .list(Some(&OwnerId::new("usr_self")), FavoriteEntityKind::Friend,)
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
    harness
        .store
        .add(
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
    let rows = harness
        .store
        .list(Some(&OwnerId::new("usr_self")), FavoriteEntityKind::Friend)
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].group_name, "Target");
    assert_single_local_invalidation(&harness.event_bus);
}

#[tokio::test]
async fn local_bulk_remove_emits_once_with_exact_changed_sides() {
    let harness = harness("local-bulk-remove");
    harness
        .store
        .add(
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
