use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestInviteRequest {
    #[serde(default)]
    pub request_slot: Option<i32>,
}
