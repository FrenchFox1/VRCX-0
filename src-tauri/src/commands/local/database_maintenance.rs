#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use serde_json::Value;
use vrcx_0_persistence::maintenance::{
    BrokenGameLogDisplayNameOutput, MaintenanceTableSizesOutput, UserTableContextOutput,
};

#[tauri::command]
#[specta::specta]
pub fn app__database_maintenance_broken_game_log_display_names_get(
    state: State<'_, AppState>,
) -> Result<Vec<BrokenGameLogDisplayNameOutput>, AppError> {
    vrcx_0_persistence::maintenance::database_maintenance_broken_game_log_display_names_get(
        state.db.as_ref(),
    )
    .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__database_maintenance_broken_leave_entries_get(
    state: State<'_, AppState>,
) -> Result<Vec<Value>, AppError> {
    vrcx_0_persistence::maintenance::database_maintenance_broken_leave_entries_get(
        state.db.as_ref(),
    )
    .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__database_maintenance_max_friend_log_number_get(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<i64, AppError> {
    vrcx_0_persistence::maintenance::database_maintenance_max_friend_log_number_get(
        state.db.as_ref(),
        user_id,
    )
    .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__database_maintenance_table_sizes_get(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<MaintenanceTableSizesOutput, AppError> {
    vrcx_0_persistence::maintenance::database_maintenance_table_sizes_get(
        state.db.as_ref(),
        user_id,
    )
    .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__user_tables_ensure(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<UserTableContextOutput, AppError> {
    vrcx_0_persistence::maintenance::user_tables_ensure(state.db.as_ref(), user_id)
        .map_err(AppError::from)
}
