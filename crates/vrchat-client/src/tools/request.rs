use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum InviteMessageType {
    Message,
    Request,
    RequestResponse,
    Response,
}

impl InviteMessageType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Request => "request",
            Self::RequestResponse => "requestResponse",
            Self::Response => "response",
        }
    }
}
