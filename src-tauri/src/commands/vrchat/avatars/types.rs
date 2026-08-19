use serde::Deserialize;
use vrcx_0_application_core::vrchat_api::avatars::{
    AvatarListSort, AvatarUpdateRequest, QueryOrder, ReleaseStatusFilter,
};
#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatAvatarIdInput {
    #[serde(default)]
    pub(crate) avatar_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatAvatarListByUserInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) user: String,
    #[serde(
        default,
        deserialize_with = "vrcx_0_application_core::vrchat_api::deserialize_nonnegative_i32"
    )]
    pub(crate) n: i32,
    #[serde(
        default,
        deserialize_with = "vrcx_0_application_core::vrchat_api::deserialize_nonnegative_i32"
    )]
    pub(crate) offset: i32,
    pub(crate) sort: AvatarListSort,
    pub(crate) order: QueryOrder,
    pub(crate) release_status: ReleaseStatusFilter,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatAvatarFileInput {
    #[serde(default)]
    pub(crate) file_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatAvatarSaveInput {
    #[serde(default)]
    pub(crate) avatar_id: String,
    pub(crate) params: AvatarUpdateRequest,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatAvatarModerationInput {
    #[serde(default)]
    pub(crate) avatar_id: String,
}
