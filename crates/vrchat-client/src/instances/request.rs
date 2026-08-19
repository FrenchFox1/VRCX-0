use serde::{Deserialize, Serialize};

use crate::http_api::{require_text, HttpApiError};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum InstanceCreateType {
    #[serde(rename = "friends")]
    Friends,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum InstanceCreateRegion {
    #[serde(rename = "eu")]
    Eu,
    #[serde(rename = "jp")]
    Jp,
    #[serde(rename = "us")]
    Us,
    #[serde(rename = "use")]
    Use,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum InstanceCreateGroupAccessType {
    #[serde(rename = "members")]
    Members,
    #[serde(rename = "plus")]
    Plus,
    #[serde(rename = "public")]
    Public,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum InstanceCreateMinimumAvatarPerformance {
    Poor,
    Medium,
    Good,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstanceCreateRequest {
    #[serde(rename = "type")]
    pub r#type: InstanceCreateType,
    pub can_request_invite: bool,
    pub world_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    pub region: InstanceCreateRegion,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_access_type: Option<InstanceCreateGroupAccessType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age_gate: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_avatar_performance: Option<InstanceCreateMinimumAvatarPerformance>,
}

impl InstanceCreateRequest {
    pub(super) fn validated(mut self) -> Result<Self, HttpApiError> {
        self.world_id = require_text(self.world_id, "VrchatInstanceCreate requires worldId.")?;
        if !self.world_id.starts_with("wrld_") {
            return Err(HttpApiError::Custom(
                "VrchatInstanceCreate requires a worldId beginning with wrld_.".into(),
            ));
        }

        self.owner_id = self
            .owner_id
            .map(crate::http_api::normalize_text)
            .filter(|owner_id| !owner_id.is_empty());

        match self.r#type {
            InstanceCreateType::Public => {
                if self.owner_id.is_some() {
                    return Err(HttpApiError::Custom(
                        "VrchatInstanceCreate public instances cannot have an ownerId.".into(),
                    ));
                }
            }
            InstanceCreateType::Group => {
                if !self
                    .owner_id
                    .as_deref()
                    .is_some_and(|owner_id| owner_id.starts_with("grp_"))
                {
                    return Err(HttpApiError::Custom(
                        "VrchatInstanceCreate group instances require a group ownerId.".into(),
                    ));
                }
            }
            InstanceCreateType::Friends
            | InstanceCreateType::Hidden
            | InstanceCreateType::Private => {
                if !self
                    .owner_id
                    .as_deref()
                    .is_some_and(|owner_id| owner_id.starts_with("usr_"))
                {
                    return Err(HttpApiError::Custom(
                        "VrchatInstanceCreate private instances require a user ownerId.".into(),
                    ));
                }
            }
        }

        if self.can_request_invite && self.r#type != InstanceCreateType::Private {
            return Err(HttpApiError::Custom(
                "VrchatInstanceCreate canRequestInvite only applies to private instances.".into(),
            ));
        }

        if self.r#type == InstanceCreateType::Group {
            let group_access_type = self.group_access_type.ok_or_else(|| {
                HttpApiError::Custom(
                    "VrchatInstanceCreate group instances require groupAccessType.".into(),
                )
            })?;
            if self.role_ids.is_some()
                && group_access_type != InstanceCreateGroupAccessType::Members
            {
                return Err(HttpApiError::Custom(
                    "VrchatInstanceCreate roleIds require members group access.".into(),
                ));
            }
            if let Some(role_ids) = &mut self.role_ids {
                for role_id in role_ids {
                    *role_id = require_text(
                        role_id.as_str(),
                        "VrchatInstanceCreate roleIds cannot contain blank values.",
                    )?;
                    if !role_id.starts_with("grol_") {
                        return Err(HttpApiError::Custom(
                            "VrchatInstanceCreate roleIds must begin with grol_.".into(),
                        ));
                    }
                }
            }
        } else if self.group_access_type.is_some()
            || self.queue_enabled.is_some()
            || self.role_ids.is_some()
            || self.age_gate.is_some()
            || self.minimum_avatar_performance.is_some()
        {
            return Err(HttpApiError::Custom(
                "VrchatInstanceCreate group options require a group instance.".into(),
            ));
        }

        Ok(self)
    }
}
