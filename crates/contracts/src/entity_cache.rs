use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vrcx_0_core::ReleaseStatus;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntityInput {
    #[serde(default)]
    pub id: Value,
    #[serde(default)]
    pub author_id: Value,
    #[serde(default)]
    pub author_name: Value,
    #[serde(default)]
    pub created_at: Value,
    #[serde(default)]
    pub description: Value,
    #[serde(default)]
    pub image_url: Value,
    #[serde(default)]
    pub name: Value,
    #[serde(default)]
    pub release_status: Value,
    #[serde(default)]
    pub thumbnail_image_url: Value,
    #[serde(default)]
    pub updated_at: Value,
    #[serde(default)]
    pub version: Value,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AvatarCacheOutput {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub description: String,
    pub image_url: String,
    pub name: String,
    #[specta(type = String)]
    pub release_status: ReleaseStatus,
    pub thumbnail_image_url: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub version: i64,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WorldSummaryOutput {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    #[serde(rename = "created_at")]
    #[specta(type = String)]
    pub created_at: CompactString,
    pub description: String,
    pub image_url: String,
    pub name: String,
    #[specta(type = String)]
    pub release_status: ReleaseStatus,
    pub thumbnail_image_url: String,
    #[serde(rename = "updated_at")]
    #[specta(type = String)]
    pub updated_at: CompactString,
    pub version: i64,
}
