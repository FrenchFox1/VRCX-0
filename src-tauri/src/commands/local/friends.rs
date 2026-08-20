#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_persistence::friends::{
    FriendLogCurrentOutput, FriendLogHistoryEntryInput, FriendLogHistoryOutput,
    FriendLogHistoryQueryInput,
};

#[tauri::command(async)]
#[specta::specta]
pub fn app__friend_log_current_list(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<Vec<FriendLogCurrentOutput>, AppError> {
    vrcx_0_persistence::friends::friend_log_current_list(state.db.as_ref(), user_id)
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__friend_log_history_delete(
    state: State<'_, AppState>,
    user_id: String,
    entry: FriendLogHistoryEntryInput,
) -> Result<i64, AppError> {
    vrcx_0_persistence::friends::friend_log_history_delete(state.db.as_ref(), user_id, entry)
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__friend_log_history_query(
    state: State<'_, AppState>,
    query: FriendLogHistoryQueryInput,
) -> Result<Vec<FriendLogHistoryOutput>, AppError> {
    vrcx_0_persistence::friends::friend_log_history_query(state.db.as_ref(), query)
        .map_err(AppError::from)
}
