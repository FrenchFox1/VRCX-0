use std::time::Duration;

use vrcx_0_application_core::{
    vrchat_api::{self, VrchatApiRequest, VrchatApiResponse, VrchatScope},
    FavoriteChange, FavoriteChangeScope, FavoriteGroupVisibility, FavoritesChangedPayload,
    RuntimeDiagnostics, RuntimeEventBus, RuntimeSyncEngine, VrchatFavoriteType, WebClient,
};
use vrcx_0_contracts::vrchat_api::parse_vrchat_json;
use vrcx_0_core::json::RawJson;

use vrcx_0_application_core::{AuthenticatedMutationContext, Result};

const FAVORITE_REMOTE_MUTATION_INTERVAL: Duration = Duration::from_millis(250);

pub(super) struct FavoriteRemoteMutationDeps<'a> {
    pub(crate) web: &'a WebClient,
    pub remote_requests: &'a dyn FavoriteRemoteRequests,
    pub diagnostics: &'a RuntimeDiagnostics,
    pub sync: &'a RuntimeSyncEngine,
    pub event_bus: &'a RuntimeEventBus,
    pub mutation: AuthenticatedMutationContext<'a>,
}

pub trait FavoriteRemoteRequests: Send + Sync {
    fn list(&self, endpoint: String, n: i32, offset: i32) -> VrchatApiRequest;
    fn limits(&self, endpoint: String) -> VrchatApiRequest;
    fn favorite_worlds(
        &self,
        endpoint: String,
        n: i32,
        offset: i32,
        owner_id: String,
        user_id: String,
        tag: String,
    ) -> VrchatApiRequest;
    fn favorite_avatars(
        &self,
        endpoint: String,
        n: i32,
        offset: i32,
        tag: String,
    ) -> VrchatApiRequest;
    fn world(&self, endpoint: String, world_id: String) -> Result<(String, VrchatApiRequest)>;
    fn avatar(&self, endpoint: String, avatar_id: String) -> Result<(String, VrchatApiRequest)>;
    fn user(&self, endpoint: String, user_id: String) -> Result<(String, VrchatApiRequest)>;
    fn add(
        &self,
        endpoint: String,
        input: FavoriteRemoteAddInput,
    ) -> Result<(String, String, VrchatApiRequest)>;
    fn delete(&self, endpoint: String, object_id: String) -> Result<(String, VrchatApiRequest)>;
    fn save_group(
        &self,
        endpoint: String,
        current_user_id: String,
        input: FavoriteRemoteGroupSaveInput,
    ) -> Result<(String, VrchatApiRequest)>;
    fn clear_group(
        &self,
        endpoint: String,
        current_user_id: String,
        input: FavoriteRemoteGroupClearInput,
    ) -> Result<(String, VrchatApiRequest)>;
}

pub struct FavoriteRemoteAddInput {
    pub kind: VrchatFavoriteType,
    pub entity_id: String,
    pub tags: String,
}

fn prepare_remote_favorite_add(
    remote_requests: &dyn FavoriteRemoteRequests,
    endpoint: String,
    input: FavoriteRemoteAddInput,
) -> Result<(FavoriteChangeScope, String, String, VrchatApiRequest)> {
    let notification_scope = input.kind.into();
    let (kind, entity_id, request) = remote_requests.add(endpoint, input)?;
    Ok((notification_scope, kind, entity_id, request))
}

pub struct FavoriteRemoteDeleteInput {
    pub object_id: String,
}

pub struct FavoriteRemoteGroupSaveInput {
    pub kind: VrchatFavoriteType,
    pub group: String,
    pub display_name: Option<String>,
    pub visibility: Option<FavoriteGroupVisibility>,
}

pub struct FavoriteRemoteGroupClearInput {
    pub kind: VrchatFavoriteType,
    pub group: String,
}

fn should_notify_favorite_change(status: i32) -> bool {
    vrchat_api::classify_api_response(status).class == vrchat_api::ApiResponseClass::Ok
}

async fn execute_remote_favorite_mutation(
    deps: &FavoriteRemoteMutationDeps<'_>,
    command: &str,
    detail: String,
    mut request: VrchatApiRequest,
) -> Result<VrchatApiResponse> {
    deps.mutation.apply_scope_to_request(&mut request);
    deps.mutation
        .run_after_wait(FAVORITE_REMOTE_MUTATION_INTERVAL, || async move {
            vrchat_api::execute_api_command(
                deps.web,
                deps.diagnostics,
                deps.sync,
                (command, detail),
                request,
                VrchatScope::Vrchat,
            )
            .await
        })
        .await
}

fn has_exact_remote_favorite_identity(favorite: &serde_json::Value) -> bool {
    ["id", "favoriteId"].iter().all(|field| {
        favorite
            .get(field)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    })
}

fn notify_favorite_change(
    deps: &FavoriteRemoteMutationDeps<'_>,
    kind: FavoriteChangeScope,
    changes: Vec<FavoriteChange>,
) {
    deps.event_bus
        .emit_favorites_changed(FavoritesChangedPayload::from_changes(
            deps.mutation.scope(),
            kind,
            false,
            true,
            changes,
        ));
}

pub(super) async fn add_remote_favorite(
    deps: &FavoriteRemoteMutationDeps<'_>,
    input: FavoriteRemoteAddInput,
) -> Result<VrchatApiResponse> {
    let (notification_scope, kind, entity_id, request) = prepare_remote_favorite_add(
        deps.remote_requests,
        deps.mutation.scope().endpoint.clone(),
        input,
    )?;
    let response = execute_remote_favorite_mutation(
        deps,
        "favorite.remote.add",
        format!("Adding {kind} favorite {entity_id}."),
        request,
    )
    .await?;
    if should_notify_favorite_change(response.status) {
        let favorite = parse_vrchat_json(&response.data);
        let changes = if has_exact_remote_favorite_identity(&favorite) {
            vec![FavoriteChange::RemoteAdded {
                favorite: RawJson::from(favorite),
            }]
        } else {
            Vec::new()
        };
        notify_favorite_change(deps, notification_scope, changes);
    }
    Ok(response)
}

pub(super) async fn delete_remote_favorite(
    deps: &FavoriteRemoteMutationDeps<'_>,
    input: FavoriteRemoteDeleteInput,
) -> Result<VrchatApiResponse> {
    let (object_id, request) = deps
        .remote_requests
        .delete(deps.mutation.scope().endpoint.clone(), input.object_id)?;
    let response = execute_remote_favorite_mutation(
        deps,
        "favorite.remote.delete",
        format!("Deleting favorite for {object_id}."),
        request,
    )
    .await?;
    if should_notify_favorite_change(response.status) {
        notify_favorite_change(
            deps,
            FavoriteChangeScope::All,
            vec![FavoriteChange::RemoteRemoved { object_id }],
        );
    }
    Ok(response)
}

pub(super) async fn save_remote_favorite_group(
    deps: &FavoriteRemoteMutationDeps<'_>,
    input: FavoriteRemoteGroupSaveInput,
) -> Result<VrchatApiResponse> {
    let notification_scope = input.kind.into();
    let (group, request) = deps.remote_requests.save_group(
        deps.mutation.scope().endpoint.clone(),
        deps.mutation.scope().current_user_id.clone(),
        input,
    )?;
    let response = execute_remote_favorite_mutation(
        deps,
        "favorite.remote.group.save",
        format!("Saving favorite group {group}."),
        request,
    )
    .await?;
    if should_notify_favorite_change(response.status) {
        notify_favorite_change(deps, notification_scope, Vec::new());
    }
    Ok(response)
}

pub(super) async fn clear_remote_favorite_group(
    deps: &FavoriteRemoteMutationDeps<'_>,
    input: FavoriteRemoteGroupClearInput,
) -> Result<VrchatApiResponse> {
    let notification_scope = input.kind.into();
    let (group, request) = deps.remote_requests.clear_group(
        deps.mutation.scope().endpoint.clone(),
        deps.mutation.scope().current_user_id.clone(),
        input,
    )?;
    let response = execute_remote_favorite_mutation(
        deps,
        "favorite.remote.group.clear",
        format!("Clearing favorite group {group}."),
        request,
    )
    .await?;
    if should_notify_favorite_change(response.status) {
        notify_favorite_change(deps, notification_scope, Vec::new());
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vrcx_0_application_core::{
        vrchat_api::VrchatApiRequest, FavoriteChangeScope, Result, VrchatFavoriteType,
    };

    use super::{
        has_exact_remote_favorite_identity, prepare_remote_favorite_add,
        should_notify_favorite_change, FavoriteRemoteAddInput, FavoriteRemoteGroupClearInput,
        FavoriteRemoteGroupSaveInput, FavoriteRemoteRequests,
    };

    struct TestFavoriteRemoteRequests;

    impl FavoriteRemoteRequests for TestFavoriteRemoteRequests {
        fn list(&self, _endpoint: String, _n: i32, _offset: i32) -> VrchatApiRequest {
            VrchatApiRequest::default()
        }

        fn limits(&self, _endpoint: String) -> VrchatApiRequest {
            VrchatApiRequest::default()
        }

        fn favorite_worlds(
            &self,
            _endpoint: String,
            _n: i32,
            _offset: i32,
            _owner_id: String,
            _user_id: String,
            _tag: String,
        ) -> VrchatApiRequest {
            VrchatApiRequest::default()
        }

        fn favorite_avatars(
            &self,
            _endpoint: String,
            _n: i32,
            _offset: i32,
            _tag: String,
        ) -> VrchatApiRequest {
            VrchatApiRequest::default()
        }

        fn world(&self, _endpoint: String, world_id: String) -> Result<(String, VrchatApiRequest)> {
            Ok((world_id, VrchatApiRequest::default()))
        }

        fn avatar(
            &self,
            _endpoint: String,
            avatar_id: String,
        ) -> Result<(String, VrchatApiRequest)> {
            Ok((avatar_id, VrchatApiRequest::default()))
        }

        fn user(&self, _endpoint: String, user_id: String) -> Result<(String, VrchatApiRequest)> {
            Ok((user_id, VrchatApiRequest::default()))
        }

        fn add(
            &self,
            _endpoint: String,
            input: FavoriteRemoteAddInput,
        ) -> Result<(String, String, VrchatApiRequest)> {
            Ok((
                input.kind.as_str().to_string(),
                input.entity_id,
                VrchatApiRequest::default(),
            ))
        }

        fn delete(
            &self,
            _endpoint: String,
            object_id: String,
        ) -> Result<(String, VrchatApiRequest)> {
            Ok((object_id, VrchatApiRequest::default()))
        }

        fn save_group(
            &self,
            _endpoint: String,
            _current_user_id: String,
            input: FavoriteRemoteGroupSaveInput,
        ) -> Result<(String, VrchatApiRequest)> {
            Ok((input.group, VrchatApiRequest::default()))
        }

        fn clear_group(
            &self,
            _endpoint: String,
            _current_user_id: String,
            input: FavoriteRemoteGroupClearInput,
        ) -> Result<(String, VrchatApiRequest)> {
            Ok((input.group, VrchatApiRequest::default()))
        }
    }

    #[test]
    fn only_successful_http_policies_are_favorite_changes() {
        for (status, expected) in [
            (200, true),
            (204, true),
            (302, false),
            (401, false),
            (429, false),
            (500, false),
        ] {
            assert_eq!(
                should_notify_favorite_change(status),
                expected,
                "status {status}"
            );
        }
    }

    #[test]
    fn vrc_plus_world_add_keeps_remote_type_and_notifies_world_scope() {
        let (scope, kind, entity_id, request) = prepare_remote_favorite_add(
            &TestFavoriteRemoteRequests,
            "endpoint".into(),
            FavoriteRemoteAddInput {
                kind: VrchatFavoriteType::VrcPlusWorld,
                entity_id: "wrld_1".into(),
                tags: "worlds4".into(),
            },
        )
        .unwrap();

        assert_eq!(scope, FavoriteChangeScope::World);
        assert_eq!(kind, "vrcPlusWorld");
        assert_eq!(entity_id, "wrld_1");
        assert_eq!(request.path, None);
        assert_eq!(request.method, None);
    }

    #[test]
    fn exact_remote_add_requires_both_favorite_identifiers() {
        assert!(has_exact_remote_favorite_identity(&json!({
            "id": "fvrt_1",
            "favoriteId": "wrld_1"
        })));
        for payload in [
            json!({}),
            json!({"id": "fvrt_1"}),
            json!({"favoriteId": "wrld_1"}),
            json!({"id": "", "favoriteId": "wrld_1"}),
            json!({"id": "fvrt_1", "favoriteId": null}),
        ] {
            assert!(!has_exact_remote_favorite_identity(&payload));
        }
    }
}
