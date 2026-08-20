#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_persistence::activity::{
    ActivityOverlapViewBuildInput, ActivityOverlapViewOutput, ActivityViewBuildInput,
    ActivityViewOutput,
};

#[tauri::command]
#[specta::specta]
pub fn app__activity_overlap_view(
    state: State<'_, AppState>,
    input: ActivityOverlapViewBuildInput,
) -> Result<ActivityOverlapViewOutput, AppError> {
    vrcx_0_persistence::activity::activity_overlap_view_build(state.db.as_ref(), input)
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__activity_view(
    state: State<'_, AppState>,
    input: ActivityViewBuildInput,
) -> Result<ActivityViewOutput, AppError> {
    vrcx_0_persistence::activity::activity_view_build(state.db.as_ref(), input)
        .map_err(AppError::from)
}
