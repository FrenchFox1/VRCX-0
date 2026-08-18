use serde::Deserialize;
use serde_json::Value;
use vrcx_0_application_core::vrchat_api::worlds::{
    QueryOrder, ReleaseStatusFilter, WorldSearchSort,
};

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatWorldIdInput {
    #[serde(default)]
    pub(crate) world_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatWorldListByUserInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) n: i64,
    #[serde(default)]
    pub(crate) offset: i64,
    pub(crate) sort: WorldSearchSort,
    pub(crate) order: QueryOrder,
    pub(crate) release_status: ReleaseStatusFilter,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatWorldSaveInput {
    #[serde(default)]
    pub(crate) world_id: String,
    pub(crate) params: Option<Value>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatWorldPersistentDataDeleteInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) world_id: String,
}
