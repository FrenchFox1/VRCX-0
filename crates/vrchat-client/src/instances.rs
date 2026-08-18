use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::http_api::{
    api_input_skip_empty_query_string as api_input, get_input_skip_empty_query_string as get_input,
    query_input, require_text, HttpApiError, HttpApiRequestInput,
};

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
}

impl InstanceCreateRequest {
    fn validated(mut self) -> Result<Self, HttpApiError> {
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
        {
            return Err(HttpApiError::Custom(
                "VrchatInstanceCreate group options require a group instance.".into(),
            ));
        }

        Ok(self)
    }
}

pub fn instance_get_input(
    endpoint: String,
    world_id: String,
    instance_id: String,
) -> Result<(String, String, HttpApiRequestInput), HttpApiError> {
    let world_id = require_text(world_id, "VrchatInstanceGet requires worldId.")?;
    let instance_id = require_text(instance_id, "VrchatInstanceGet requires instanceId.")?;
    Ok((
        world_id.clone(),
        instance_id.clone(),
        get_input(
            endpoint,
            format!("instances/{world_id}:{instance_id}"),
            HashMap::new(),
        ),
    ))
}

pub fn instance_short_name_get_input(
    endpoint: String,
    world_id: String,
    instance_id: String,
    short_name: String,
) -> Result<(String, String, HttpApiRequestInput), HttpApiError> {
    let world_id = require_text(world_id, "VrchatInstanceShortNameGet requires worldId.")?;
    let instance_id = require_text(
        instance_id,
        "VrchatInstanceShortNameGet requires instanceId.",
    )?;
    let mut params = HashMap::new();
    if !short_name.is_empty() {
        params.insert("shortName".to_string(), Value::String(short_name));
    }
    Ok((
        world_id.clone(),
        instance_id.clone(),
        get_input(
            endpoint,
            format!("instances/{world_id}:{instance_id}/shortName"),
            params,
        ),
    ))
}

pub fn instance_create_input(
    endpoint: String,
    params: InstanceCreateRequest,
) -> Result<HttpApiRequestInput, HttpApiError> {
    let params = params.validated()?;
    Ok(api_input(endpoint, "POST", "instances", json!(params)))
}

pub fn instance_self_invite_input(
    endpoint: String,
    world_id: String,
    instance_id: String,
    short_name: String,
) -> Result<(String, String, HttpApiRequestInput), HttpApiError> {
    let world_id = require_text(world_id, "VrchatInstanceSelfInvite requires worldId.")?;
    let instance_id = require_text(instance_id, "VrchatInstanceSelfInvite requires instanceId.")?;
    let body = if short_name.is_empty() {
        HashMap::new()
    } else {
        HashMap::from([("shortName".to_string(), Value::String(short_name))])
    };
    Ok((
        world_id.clone(),
        instance_id.clone(),
        query_input(
            endpoint,
            "POST",
            format!("invite/myself/to/{world_id}:{instance_id}"),
            body,
        ),
    ))
}

pub fn instance_close_input(
    endpoint: String,
    location: String,
    hard_close: bool,
) -> Result<(String, HttpApiRequestInput), HttpApiError> {
    let location = require_text(location, "VrchatInstanceClose requires location.")?;
    Ok((
        location.clone(),
        api_input(
            endpoint,
            "DELETE",
            format!("instances/{location}"),
            json!({ "hardClose": hard_close }),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public_instance() -> InstanceCreateRequest {
        InstanceCreateRequest {
            r#type: InstanceCreateType::Public,
            can_request_invite: false,
            world_id: "wrld_123".into(),
            owner_id: None,
            region: InstanceCreateRegion::Us,
            group_access_type: None,
            queue_enabled: None,
            role_ids: None,
            age_gate: None,
            display_name: None,
        }
    }

    #[test]
    fn create_instance_serializes_only_valid_typed_options() {
        let request = instance_create_input("endpoint".into(), public_instance()).unwrap();

        assert_eq!(
            request.body.as_json(),
            Some(&json!({
                "type": "public",
                "canRequestInvite": false,
                "worldId": "wrld_123",
                "region": "us",
            }))
        );
    }

    #[test]
    fn create_instance_rejects_cross_field_mismatches() {
        let mut request = public_instance();
        request.owner_id = Some("usr_owner".into());
        assert!(instance_create_input("endpoint".into(), request).is_err());

        let mut request = public_instance();
        request.r#type = InstanceCreateType::Group;
        request.owner_id = Some("grp_owner".into());
        request.group_access_type = Some(InstanceCreateGroupAccessType::Plus);
        request.role_ids = Some(vec!["grol_role".into()]);
        assert!(instance_create_input("endpoint".into(), request).is_err());

        let mut request = public_instance();
        request.can_request_invite = true;
        assert!(instance_create_input("endpoint".into(), request).is_err());
    }

    #[test]
    fn short_name_lookup_keeps_instance_tag_unescaped_like_legacy_api() {
        let (_, _, request) = instance_short_name_get_input(
            "".into(),
            "wrld_123".into(),
            "12345~hidden(usr_owner)".into(),
            "".into(),
        )
        .unwrap();

        assert_eq!(
            request.path.as_deref(),
            Some("instances/wrld_123:12345~hidden(usr_owner)/shortName")
        );
    }

    #[test]
    fn self_invite_uses_short_name_as_query_param_without_json_body() {
        let (_, _, request) = instance_self_invite_input(
            "".into(),
            "wrld_123".into(),
            "12345~hidden(usr_owner)".into(),
            "abc123".into(),
        )
        .unwrap();

        assert_eq!(
            request.path.as_deref(),
            Some("invite/myself/to/wrld_123:12345~hidden(usr_owner)")
        );
        assert_eq!(request.method.as_deref(), Some("POST"));
        assert_eq!(request.body, crate::http_api::HttpApiRequestBody::Empty);
        assert_eq!(
            request
                .query_params
                .as_ref()
                .and_then(|params| params.get("shortName")),
            Some(&Value::String("abc123".into()))
        );
    }
}
