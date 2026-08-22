#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application_core::vrchat_api::{VrchatApiRequest, VrchatApiResponse, VrchatScope};

use crate::error::AppError;
use crate::state::AppState;

pub async fn execute_vrchat_api(
    state: State<'_, AppState>,
    command: &str,
    detail: impl Into<String>,
    input: VrchatApiRequest,
    scope: VrchatScope,
) -> Result<VrchatApiResponse, AppError> {
    state
        .runtime_host()
        .vrchat_api()
        .execute(command, detail, input, scope)
        .await
        .map_err(AppError::from)
}
