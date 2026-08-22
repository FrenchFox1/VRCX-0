#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_application::favorites::{FavoriteRow, LocalFavoriteSnapshot};
use vrcx_0_application_core::FavoriteEntityKind;

#[tauri::command(async)]
#[specta::specta]
pub fn app__favorite_list(
    state: State<'_, AppState>,
    kind: FavoriteEntityKind,
) -> Result<Vec<FavoriteRow>, AppError> {
    state
        .runtime_host()
        .local_data()
        .favorite_list(kind)
        .map_err(AppError::from)
}

#[tauri::command(async)]
#[specta::specta]
pub fn app__favorite_local_snapshot(
    state: State<'_, AppState>,
    kind: FavoriteEntityKind,
) -> Result<LocalFavoriteSnapshot, AppError> {
    state
        .runtime_host()
        .local_data()
        .favorite_snapshot(kind)
        .map_err(AppError::from)
}

pub fn favorite_add(
    state: State<'_, AppState>,
    kind: FavoriteEntityKind,
    entity_id: String,
    group_name: String,
) -> Result<i64, AppError> {
    state
        .runtime_host()
        .local_data()
        .favorite_add_local(kind, entity_id, group_name)
        .map_err(AppError::from)
}

pub fn favorite_remove(
    state: State<'_, AppState>,
    kind: FavoriteEntityKind,
    entity_id: String,
    group_name: String,
) -> Result<i64, AppError> {
    state
        .runtime_host()
        .local_data()
        .favorite_remove_local(kind, entity_id, group_name)
        .map_err(AppError::from)
}
