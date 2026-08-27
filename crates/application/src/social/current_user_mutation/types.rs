use serde::{Deserialize, Serialize};
use vrcx_0_core::friends::UserStatus;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum ContentFilter {
    #[serde(rename = "content_adult")]
    Adult,
    #[serde(rename = "content_gore")]
    Gore,
    #[serde(rename = "content_horror")]
    Horror,
    #[serde(rename = "content_sex")]
    Sex,
    #[serde(rename = "content_violence")]
    Violence,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "backgroundType", rename_all = "camelCase", deny_unknown_fields)]
pub enum CurrentUserProfileUpdateRequest {
    Default,
    Gradient {
        #[serde(rename = "backgroundGradientBottom")]
        background_gradient_bottom: String,
        #[serde(rename = "backgroundGradientTop")]
        background_gradient_top: String,
    },
    Texture {
        #[serde(rename = "backgroundTextureId")]
        background_texture_id: String,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurrentUserUpdateRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio_links: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_pic_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_avatar_copying: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_booping_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_shared_connections_opt_out: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_discord_friends_opt_out: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_filters: Option<Vec<ContentFilter>>,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserProfileUpdateInput {
    pub params: CurrentUserProfileUpdateRequest,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserUpdateInput {
    pub params: CurrentUserUpdateRequest,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserBadgeInput {
    #[serde(default)]
    pub badge_id: String,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub showcased: bool,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatCurrentUserTagsInput {
    #[serde(default)]
    pub tags: Vec<String>,
}
