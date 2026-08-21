#![allow(non_snake_case)]

use serde::Deserialize;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_application_core::vrchat_api::VrchatApiResponse;
use vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT;

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorldGetInput {
    #[serde(default)]
    world_id: String,
    #[serde(default)]
    force: bool,
    #[serde(default)]
    full: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn app__world_get(
    state: State<'_, AppState>,
    input: WorldGetInput,
) -> Result<VrchatApiResponse, AppError> {
    let auth_scope = state.runtime_context.auth_scope.snapshot();
    let endpoint = if auth_scope.endpoint.is_empty() {
        VRCHAT_API_DEFAULT_ENDPOINT
    } else {
        auth_scope.endpoint.as_str()
    };
    state
        .runtime_context
        .world_cache
        .get(
            state.web.as_ref(),
            endpoint,
            &input.world_id,
            input.force,
            input.full,
        )
        .await
        .map_err(AppError::from)
}
