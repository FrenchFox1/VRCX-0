#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::profile::{AppUpdateDownloadStatusSnapshot, AppUpdateStatusSnapshot};
use vrcx_0_application_core::UpdaterMetadata;

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__app_update_check_run(
    state: State<'_, AppState>,
) -> Result<AppUpdateStatusSnapshot, AppError> {
    Ok(state.runtime_host().check_for_app_update().await)
}

#[tauri::command]
#[specta::specta]
pub fn app__app_update_download_status_get(
    state: State<'_, AppState>,
) -> AppUpdateDownloadStatusSnapshot {
    state.runtime_host().app_update_download_status()
}

#[tauri::command]
#[specta::specta]
pub async fn app__app_update_install_confirm(
    state: State<'_, AppState>,
    version: String,
) -> Result<UpdaterMetadata, AppError> {
    state
        .runtime_host()
        .install_app_update(&version)
        .await
        .map_err(AppError::from)
}
