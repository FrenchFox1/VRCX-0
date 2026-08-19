use serde::Deserialize;
use vrcx_0_application_core::vrchat_api::users::{
    CurrentUserProfileUpdateRequest, CurrentUserUpdateRequest,
};

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatUserInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) force: bool,
    #[serde(default)]
    pub(crate) dialog: bool,
    #[serde(default)]
    pub(crate) is_friend: Option<bool>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatUserProfileInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) as_self: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserProfileUpdateInput {
    pub(crate) params: CurrentUserProfileUpdateRequest,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserUpdateInput {
    pub(crate) params: CurrentUserUpdateRequest,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserBadgeInput {
    #[serde(default)]
    pub(crate) badge_id: String,
    #[serde(default)]
    pub(crate) hidden: bool,
    #[serde(default)]
    pub(crate) showcased: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserTagsInput {
    #[serde(default)]
    pub(crate) tags: Vec<String>,
}
