#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application_core::vrchat_api::{VrchatApiRequest, VrchatApiResponse, VrchatScope};
use vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT;
use vrcx_0_vrchat_client::avatars::{
    avatar_file_get_input, avatar_gallery_get_input, avatar_list_by_user_get_input,
    avatar_styles_get_input, AvatarListByUserGetInput,
};

use crate::error::AppError;
use crate::state::AppState;

use super::types::{
    VrchatAvatarFileInput, VrchatAvatarIdInput, VrchatAvatarListByUserInput,
    VrchatAvatarModerationInput, VrchatAvatarSaveInput,
};

async fn execute_avatar_api(
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
pub async fn app__vrchat_avatar_gallery_get(
    state: State<'_, AppState>,
    input: VrchatAvatarIdInput,
) -> Result<VrchatApiResponse, AppError> {
    let (avatar_id, request) =
        avatar_gallery_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), input.avatar_id)?;
    execute_avatar_api(
        state,
        "app__vrchat_avatar_gallery_get",
        format!("Getting avatar gallery for {avatar_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_list_by_user_get(
    state: State<'_, AppState>,
    input: VrchatAvatarListByUserInput,
) -> Result<VrchatApiResponse, AppError> {
    let (display_user, request) = avatar_list_by_user_get_input(AvatarListByUserGetInput {
        endpoint: VRCHAT_API_DEFAULT_ENDPOINT.into(),
        user_id: input.user_id,
        user: input.user,
        n: input.n,
        offset: input.offset,
        sort: input.sort,
        order: input.order,
        release_status: input.release_status,
    })?;
    execute_avatar_api(
        state,
        "app__vrchat_avatar_list_by_user_get",
        format!("Getting avatars for {display_user}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_styles_get(
    state: State<'_, AppState>,
) -> Result<VrchatApiResponse, AppError> {
    execute_avatar_api(
        state,
        "app__vrchat_avatar_styles_get",
        "Getting avatar styles.",
        avatar_styles_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into()),
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_moderations_get(
    state: State<'_, AppState>,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state.runtime_host().avatars().moderations().await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_file_get(
    state: State<'_, AppState>,
    input: VrchatAvatarFileInput,
) -> Result<VrchatApiResponse, AppError> {
    let (file_id, request) =
        avatar_file_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), input.file_id)?;
    execute_avatar_api(
        state,
        "app__vrchat_avatar_file_get",
        format!("Getting file {file_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_select(
    state: State<'_, AppState>,
    input: VrchatAvatarIdInput,
) -> Result<vrcx_0_application::avatars::AvatarSelectionMutationOutcome, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .select(input.avatar_id)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_select_fallback(
    state: State<'_, AppState>,
    input: VrchatAvatarIdInput,
) -> Result<vrcx_0_application::avatars::AvatarSelectionMutationOutcome, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .select_fallback(input.avatar_id)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_save(
    state: State<'_, AppState>,
    input: VrchatAvatarSaveInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .save(input.avatar_id, input.params)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_delete(
    state: State<'_, AppState>,
    input: VrchatAvatarIdInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .delete(input.avatar_id)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_impostor_create(
    state: State<'_, AppState>,
    input: VrchatAvatarIdInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .create_impostor(input.avatar_id)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_impostor_delete(
    state: State<'_, AppState>,
    input: VrchatAvatarIdInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .delete_impostor(input.avatar_id)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_moderation_send(
    state: State<'_, AppState>,
    input: VrchatAvatarModerationInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .send_moderation(input.avatar_id)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_avatar_moderation_delete(
    state: State<'_, AppState>,
    input: VrchatAvatarModerationInput,
) -> Result<VrchatApiResponse, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .delete_moderation(input.avatar_id)
        .await?)
}
