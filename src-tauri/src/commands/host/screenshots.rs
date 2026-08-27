#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;
use vrcx_0_core::screenshots::{
    ScreenshotFolderTree, ScreenshotLibraryImage, ScreenshotLibraryScanStatus,
    ScreenshotSearchResult,
};

use vrcx_0_host_desktop::host_capabilities::{require_host_capability, HostCapability};

fn ensure_screenshot_read_allowed(state: &AppState, path: &str) -> Result<(), AppError> {
    state
        .runtime_host()
        .screenshots()
        .ensure_read_allowed(path)?;
    Ok(())
}

fn ensure_screenshot_write_allowed(state: &AppState, path: &str) -> Result<(), AppError> {
    state
        .runtime_host()
        .screenshots()
        .ensure_write_allowed(path)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn app__get_extra_screenshot_data(
    state: State<'_, AppState>,
    path: String,
    carousel_cache: bool,
) -> Result<String, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    ensure_screenshot_read_allowed(&state, &path)?;
    Ok(state
        .runtime_host()
        .screenshots()
        .extra_data(&path, carousel_cache)?)
}

#[tauri::command]
#[specta::specta]
pub fn app__get_screenshot_metadata(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    ensure_screenshot_read_allowed(&state, &path)?;
    Ok(state.runtime_host().screenshots().metadata_json(&path)?)
}

#[tauri::command]
#[specta::specta]
pub fn app__find_screenshots_by_search(
    state: State<'_, AppState>,
    search_query: String,
    search_type: Option<i32>,
) -> Result<Vec<ScreenshotSearchResult>, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state
        .runtime_host()
        .screenshots()
        .find(&search_query, search_type))
}

#[tauri::command]
#[specta::specta]
pub fn app__start_screenshot_library_scan(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<ScreenshotLibraryScanStatus, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state
        .runtime_host()
        .start_screenshot_library_scan(force.unwrap_or(false)))
}

#[tauri::command]
#[specta::specta]
pub fn app__get_screenshot_library_status(
    state: State<'_, AppState>,
) -> Result<ScreenshotLibraryScanStatus, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state.runtime_host().screenshots().scan_status())
}

#[tauri::command]
#[specta::specta]
pub fn app__get_screenshot_folder_tree(
    state: State<'_, AppState>,
) -> Result<ScreenshotFolderTree, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state.runtime_host().screenshots().folder_tree()?)
}

#[tauri::command]
#[specta::specta]
pub fn app__get_screenshot_folder_images(
    state: State<'_, AppState>,
    folder_path: String,
) -> Result<Vec<ScreenshotLibraryImage>, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state
        .runtime_host()
        .screenshots()
        .folder_images(&folder_path)?)
}

#[tauri::command]
#[specta::specta]
pub fn app__get_world_screenshots(
    state: State<'_, AppState>,
    world_id: String,
) -> Result<Vec<ScreenshotLibraryImage>, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state
        .runtime_host()
        .screenshots()
        .world_screenshots(&world_id)?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__ensure_screenshot_thumbnail(
    state: State<'_, AppState>,
    path: String,
) -> Result<String, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    ensure_screenshot_read_allowed(&state, &path)?;
    let screenshots = state.runtime_host().screenshots().clone();
    Ok(
        tauri::async_runtime::spawn_blocking(move || screenshots.ensure_thumbnail(&path))
            .await
            .map_err(|error| AppError::Custom(format!("thumbnail task failed: {error}")))??,
    )
}

#[tauri::command]
#[specta::specta]
pub fn app__get_last_screenshot(state: State<'_, AppState>) -> Result<String, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    Ok(state.runtime_host().screenshots().last())
}

#[tauri::command]
#[specta::specta]
pub fn app__delete_screenshot_metadata(
    state: State<'_, AppState>,
    path: String,
) -> Result<bool, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    ensure_screenshot_write_allowed(&state, &path)?;
    Ok(state.runtime_host().screenshots().delete_metadata(&path))
}

#[tauri::command]
#[specta::specta]
pub async fn app__delete_screenshot_file(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    let screenshots = state.runtime_host().screenshots().clone();
    tauri::async_runtime::spawn_blocking(move || screenshots.delete_file(&path))
        .await
        .map_err(|error| AppError::Custom(format!("screenshot delete task failed: {error}")))??;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn app__delete_all_screenshot_metadata(state: State<'_, AppState>) -> Result<(), AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    state.runtime_host().screenshots().delete_all_metadata();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn app__add_screenshot_metadata(
    state: State<'_, AppState>,
    path: String,
    metadata_string: String,
    world_id: String,
    change_filename: Option<bool>,
) -> Result<String, AppError> {
    require_host_capability(HostCapability::ScreenshotCache)?;
    ensure_screenshot_write_allowed(&state, &path)?;
    let screenshots = state.runtime_host().screenshots().clone();
    tauri::async_runtime::spawn_blocking(move || {
        screenshots.add_metadata(
            &path,
            &metadata_string,
            &world_id,
            change_filename.unwrap_or(false),
        )
    })
    .await
    .map_err(|error| AppError::Custom(format!("screenshot metadata task failed: {error}")))
}
