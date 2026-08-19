use serde::{Deserialize, Serialize};

use crate::http_api::{require_text, HttpApiError};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum GroupPostVisibility {
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "public")]
    Public,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupPostMutation {
    pub title: String,
    pub text: String,
    pub send_notification: bool,
    pub visibility: GroupPostVisibility,
    #[serde(default)]
    pub role_ids: Vec<String>,
    pub image_id: Option<String>,
}

impl GroupPostMutation {
    pub(super) fn validated(mut self) -> Result<Self, HttpApiError> {
        self.title = require_text(self.title, "VrchatGroupPost requires title.")?;
        self.text = require_text(self.text, "VrchatGroupPost requires text.")?;
        if self.visibility == GroupPostVisibility::Public && !self.role_ids.is_empty() {
            return Err(HttpApiError::Custom(
                "VrchatGroupPost roleIds require group visibility.".into(),
            ));
        }
        for role_id in &mut self.role_ids {
            *role_id = require_text(
                role_id.as_str(),
                "VrchatGroupPost roleIds cannot contain blank values.",
            )?;
            if !role_id.starts_with("grol_") {
                return Err(HttpApiError::Custom(
                    "VrchatGroupPost roleIds must begin with grol_.".into(),
                ));
            }
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum GroupMemberVisibility {
    #[serde(rename = "friends")]
    Friends,
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "visible")]
    Visible,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupMemberPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_subscribed_to_announcements: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_subscribed_to_event_announcements: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manager_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<GroupMemberVisibility>,
}
