#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_runtime_host_desktop::local_data::{
    ActivityOverlapViewBuildInput, ActivityOverlapViewOutput, ActivityViewBuildInput,
    ActivityViewOutput,
};

#[tauri::command]
#[specta::specta]
pub fn app__activity_overlap_view(
    state: State<'_, AppState>,
    input: ActivityOverlapViewBuildInput,
) -> Result<ActivityOverlapViewOutput, AppError> {
    state
        .runtime_host()
        .local_data()
        .activity_overlap_view(input)
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__activity_view(
    state: State<'_, AppState>,
    input: ActivityViewBuildInput,
) -> Result<ActivityViewOutput, AppError> {
    state
        .runtime_host()
        .local_data()
        .activity_view(input)
        .map_err(AppError::from)
}
