#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT;
use vrcx_0_vrchat_client::users::{profile_get_input, user_represented_group_get_input};

use crate::error::AppError;
use crate::state::AppState;
use vrcx_0_application_core::vrchat_api::{VrchatApiRequest, VrchatApiResponse, VrchatScope};

use super::types::{
    VrchatCurrentUserBadgeInput, VrchatCurrentUserProfileUpdateInput, VrchatCurrentUserTagsInput,
    VrchatCurrentUserUpdateInput, VrchatUserInput, VrchatUserProfileInput,
};

async fn execute_user_read_api(
    state: State<'_, AppState>,
    command: &str,
    detail: impl Into<String>,
    input: VrchatApiRequest,
) -> Result<VrchatApiResponse, AppError> {
    super::super::execute::execute_vrchat_api(state, command, detail, input, VrchatScope::Vrchat)
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_user_profile_get(
    state: State<'_, AppState>,
    input: VrchatUserProfileInput,
) -> Result<VrchatApiResponse, AppError> {
    let (user_id, request) = profile_get_input(
        VRCHAT_API_DEFAULT_ENDPOINT.into(),
        input.user_id,
        input.as_self,
    )?;
    execute_user_read_api(
        state,
        "app__vrchat_user_profile_get",
        format!("Getting profile for user {user_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_current_user_profile_update(
    state: State<'_, AppState>,
    input: VrchatCurrentUserProfileUpdateInput,
) -> Result<VrchatApiResponse, AppError> {
    state
        .runtime_host()
        .current_user_mutations()
        .update_profile(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_user_get(
    state: State<'_, AppState>,
    input: VrchatUserInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .get_user_via_cache(input.user_id, input.force, input.dialog, input.is_friend)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_user_represented_group_get(
    state: State<'_, AppState>,
    input: VrchatUserInput,
) -> Result<VrchatApiResponse, AppError> {
    let (user_id, request) =
        user_represented_group_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), input.user_id)?;
    execute_user_read_api(
        state,
        "app__vrchat_user_represented_group_get",
        format!("Getting represented group for user {user_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_current_user_update(
    state: State<'_, AppState>,
    input: VrchatCurrentUserUpdateInput,
) -> Result<VrchatApiResponse, AppError> {
    state
        .runtime_host()
        .current_user_mutations()
        .update_user(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_current_user_badge_update(
    state: State<'_, AppState>,
    input: VrchatCurrentUserBadgeInput,
) -> Result<VrchatApiResponse, AppError> {
    state
        .runtime_host()
        .current_user_mutations()
        .update_badge(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_current_user_tags_add(
    state: State<'_, AppState>,
    input: VrchatCurrentUserTagsInput,
) -> Result<VrchatApiResponse, AppError> {
    state
        .runtime_host()
        .current_user_mutations()
        .add_tags(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_current_user_tags_remove(
    state: State<'_, AppState>,
    input: VrchatCurrentUserTagsInput,
) -> Result<VrchatApiResponse, AppError> {
    state
        .runtime_host()
        .current_user_mutations()
        .remove_tags(input)
        .await
        .map_err(AppError::from)
}
