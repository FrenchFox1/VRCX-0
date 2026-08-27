#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::game::{InstanceLaunchInput, InstanceLaunchOutcome};
use vrcx_0_application_core::vrchat_api::VrchatScope;
use vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT;
use vrcx_0_runtime_host_desktop::vrchat_api::protocol::instances::{
    instance_close_input, instance_create_input, instance_get_input, instance_self_invite_input,
    instance_short_name_get_input,
};

use crate::error::AppError;
use crate::state::AppState;
use vrcx_0_application_core::vrchat_api::{VrchatApiRequest, VrchatApiResponse};

use super::types::{
    VrchatInstanceCloseInput, VrchatInstanceCreateInput, VrchatInstanceIdentityInput,
    VrchatInstanceSelfInviteInput, VrchatInstanceShortNameInput,
};

async fn execute_instance_api(
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
pub async fn app__vrchat_instance_get(
    state: State<'_, AppState>,
    input: VrchatInstanceIdentityInput,
) -> Result<VrchatApiResponse, AppError> {
    let (world_id, instance_id, request) = instance_get_input(
        VRCHAT_API_DEFAULT_ENDPOINT.into(),
        input.world_id,
        input.instance_id,
    )?;
    execute_instance_api(
        state,
        "app__vrchat_instance_get",
        format!("Getting instance {world_id}:{instance_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_instance_short_name_get(
    state: State<'_, AppState>,
    input: VrchatInstanceShortNameInput,
) -> Result<VrchatApiResponse, AppError> {
    let (world_id, instance_id, request) = instance_short_name_get_input(
        VRCHAT_API_DEFAULT_ENDPOINT.into(),
        input.world_id,
        input.instance_id,
        input.short_name,
    )?;
    execute_instance_api(
        state,
        "app__vrchat_instance_short_name_get",
        format!("Getting short name for instance {world_id}:{instance_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_instance_create(
    state: State<'_, AppState>,
    input: VrchatInstanceCreateInput,
) -> Result<VrchatApiResponse, AppError> {
    execute_instance_api(
        state,
        "app__vrchat_instance_create",
        "Creating instance.",
        instance_create_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), input.params)?,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_instance_self_invite(
    state: State<'_, AppState>,
    input: VrchatInstanceSelfInviteInput,
) -> Result<VrchatApiResponse, AppError> {
    let (world_id, instance_id, request) = instance_self_invite_input(
        VRCHAT_API_DEFAULT_ENDPOINT.into(),
        input.world_id,
        input.instance_id,
        input.short_name,
    )?;
    execute_instance_api(
        state,
        "app__vrchat_instance_self_invite",
        format!("Sending self invite for {world_id}:{instance_id}."),
        request,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_instance_join(
    state: State<'_, AppState>,
    input: InstanceLaunchInput,
) -> Result<InstanceLaunchOutcome, AppError> {
    state
        .runtime_host()
        .instance_launch()
        .join(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__vrchat_instance_close(
    state: State<'_, AppState>,
    input: VrchatInstanceCloseInput,
) -> Result<VrchatApiResponse, AppError> {
    let (location, request) = instance_close_input(
        VRCHAT_API_DEFAULT_ENDPOINT.into(),
        input.location,
        input.hard_close,
    )?;
    execute_instance_api(
        state,
        "app__vrchat_instance_close",
        format!("Closing instance {location}."),
        request,
    )
    .await
}
