#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_persistence::local_moderation::LocalModerationOutput;
use vrcx_0_persistence::OwnerId;

#[tauri::command(async)]
#[specta::specta]
pub fn app__local_moderation_get(
    state: State<'_, AppState>,
    owner_user_id: OwnerId,
    user_id: String,
) -> Result<Option<LocalModerationOutput>, AppError> {
    vrcx_0_persistence::local_moderation::local_moderation_get(
        state.db.as_ref(),
        owner_user_id,
        user_id,
    )
    .map_err(AppError::from)
}

#[tauri::command(async)]
#[specta::specta]
pub fn app__local_moderation_list(
    state: State<'_, AppState>,
    owner_user_id: OwnerId,
) -> Result<Vec<LocalModerationOutput>, AppError> {
    vrcx_0_persistence::local_moderation::local_moderation_list(state.db.as_ref(), owner_user_id)
        .map_err(AppError::from)
}
