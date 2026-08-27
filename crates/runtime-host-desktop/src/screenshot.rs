use std::path::Path;

use vrcx_0_outbound_adapters::screenshots::{
    self as screenshot, ScreenshotFolderTree, ScreenshotLibraryImage, ScreenshotLibraryScanStatus,
    ScreenshotSearchResult,
};
use vrcx_0_persistence::screenshot_cache::MetadataCacheDb;
use vrcx_0_platform::app_paths::AppPaths;

use crate::{HostFileAccess, Result};

#[derive(Clone)]
pub struct DesktopScreenshotRuntime {
    cache: MetadataCacheDb,
    host_file_access: HostFileAccess,
    paths: AppPaths,
    photos_root: String,
}

impl DesktopScreenshotRuntime {
    pub(crate) fn new(
        cache: MetadataCacheDb,
        host_file_access: HostFileAccess,
        paths: AppPaths,
        photos_root: String,
    ) -> Self {
        Self {
            cache,
            host_file_access,
            paths,
            photos_root,
        }
    }

    pub fn ensure_read_allowed(&self, path: &str) -> Result<()> {
        self.host_file_access
            .ensure_read_allowed(path, &self.paths)?;
        self.ensure_managed_screenshot(path)
    }

    pub fn ensure_delete_allowed(&self, path: &str) -> Result<()> {
        self.host_file_access
            .ensure_write_allowed(path, &self.paths)?;
        self.ensure_managed_screenshot(path)
    }

    fn ensure_managed_screenshot(&self, path: &str) -> Result<()> {
        if !screenshot::is_managed_screenshot_file_path(path, &self.photos_root) {
            return Err(crate::Error::Custom(
                "Screenshot metadata commands require a VRChat screenshot or library PNG path."
                    .into(),
            ));
        }
        Ok(())
    }

    pub fn ensure_write_allowed(&self, path: &str) -> Result<()> {
        self.host_file_access
            .ensure_write_allowed(path, &self.paths)?;
        if !screenshot::is_vrchat_screenshot_file_path(Path::new(path)) {
            return Err(crate::Error::Custom(
                "Screenshot metadata commands require a VRChat PNG screenshot path.".into(),
            ));
        }
        Ok(())
    }

    pub fn extra_data(&self, path: &str, carousel_cache: bool) -> Result<String> {
        Ok(screenshot::extra_screenshot_data(
            path,
            carousel_cache,
            &self.cache,
            &self.photos_root,
        )?)
    }

    pub fn metadata_json(&self, path: &str) -> Result<String> {
        Ok(screenshot::screenshot_metadata_json(path)?)
    }

    pub fn last(&self) -> String {
        screenshot::last_screenshot(&self.photos_root)
    }

    pub fn delete_metadata(&self, path: &str) -> bool {
        screenshot::delete_text_metadata(path, true)
    }

    pub fn delete_file(&self, path: &str) -> Result<()> {
        self.ensure_delete_allowed(path)?;
        vrcx_0_host_desktop::shell_actions::move_to_trash(Path::new(path))?;
        screenshot::forget_screenshot_file(&self.cache, &self.paths.screenshot_thumbs, path)?;
        Ok(())
    }

    pub fn add_metadata(
        &self,
        path: &str,
        metadata: &str,
        world_id: &str,
        change_filename: bool,
    ) -> String {
        screenshot::add_screenshot_metadata(path, metadata, world_id, change_filename)
    }

    pub fn find(
        &self,
        search_query: &str,
        search_type: Option<i32>,
    ) -> Vec<ScreenshotSearchResult> {
        screenshot::find_screenshot_search_results(
            search_query,
            search_type,
            &self.cache,
            &self.photos_root,
        )
    }

    pub fn scan_status(&self) -> ScreenshotLibraryScanStatus {
        self.cache.scan_status()
    }

    pub fn folder_tree(&self) -> Result<ScreenshotFolderTree> {
        Ok(screenshot::screenshot_folder_tree(
            &self.cache,
            &self.photos_root,
        )?)
    }

    pub fn folder_images(&self, folder_path: &str) -> Result<Vec<ScreenshotLibraryImage>> {
        Ok(screenshot::list_screenshot_folder_images(
            &self.cache,
            folder_path,
            &self.photos_root,
        )?)
    }

    pub fn world_screenshots(&self, world_id: &str) -> Result<Vec<ScreenshotLibraryImage>> {
        Ok(screenshot::list_world_screenshots(
            &self.cache,
            world_id,
            &self.photos_root,
        )?)
    }

    pub fn ensure_thumbnail(&self, path: &str) -> Result<String> {
        Ok(screenshot::ensure_screenshot_thumbnail(
            path,
            &self.paths.screenshot_thumbs,
            &self.cache,
            &self.photos_root,
        )?)
    }

    pub fn delete_all_metadata(&self) {
        screenshot::delete_all_screenshot_metadata(
            &self.cache,
            &self.paths.screenshot_thumbs,
            &self.photos_root,
        );
    }
}
