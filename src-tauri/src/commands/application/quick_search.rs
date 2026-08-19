#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::{QuickSearchQueryInput, QuickSearchQueryOutput};

use crate::{error::AppError, state::AppState};

#[tauri::command]
#[specta::specta]
pub async fn app__quick_search_query(
    state: State<'_, AppState>,
    input: QuickSearchQueryInput,
) -> Result<QuickSearchQueryOutput, AppError> {
    state
        .quick_search
        .query(input, state.realtime_runtime.friend_snapshot())
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub fn app__quick_search_working_set_invalidate(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.quick_search.invalidate_remote_working_set();
    Ok(())
}
