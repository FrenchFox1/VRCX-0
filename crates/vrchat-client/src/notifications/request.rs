use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestInviteRequest {
    pub platform: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_slot: Option<i64>,
}
