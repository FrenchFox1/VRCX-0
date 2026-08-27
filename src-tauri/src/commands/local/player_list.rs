#![allow(non_snake_case)]

use std::collections::HashMap;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_application_game::PlayerListSnapshotOutput;
use vrcx_0_runtime_host_desktop::local_data::{InstanceActivityRowOutput, WorldSummaryOutput};

#[tauri::command(async)]
#[specta::specta]
pub fn app__player_list_current_snapshot(
    state: State<'_, AppState>,
    current_user_id: String,
    current_location: String,
    current_location_started_at: String,
) -> Result<PlayerListSnapshotOutput, AppError> {
    state
        .runtime_host()
        .local_data()
        .player_list_current_snapshot(
            current_user_id,
            current_location,
            current_location_started_at,
        )
        .map_err(AppError::from)
}

#[tauri::command(async)]
#[specta::specta]
pub fn app__instance_activity_dates_get(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<Vec<String>, AppError> {
    state
        .runtime_host()
        .local_data()
        .instance_activity_dates_get(user_id)
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__instance_activity_rows_get(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<InstanceActivityRowOutput>, AppError> {
    state
        .runtime_host()
        .local_data()
        .instance_activity_rows_get(start_date, end_date)
        .map_err(AppError::from)
}

#[tauri::command(async)]
#[specta::specta]
pub fn app__world_summaries_get(
    state: State<'_, AppState>,
    world_ids: Vec<String>,
) -> Result<HashMap<String, WorldSummaryOutput>, AppError> {
    state
        .runtime_host()
        .local_data()
        .world_summaries_get(world_ids)
        .map_err(AppError::from)
}
