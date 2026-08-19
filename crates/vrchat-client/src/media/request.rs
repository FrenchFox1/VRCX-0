use serde::{Deserialize, Serialize};

use super::{EmojiUploadParams, PrintUploadParams};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ProfileDecorationEquipSlot {
    IconFrame,
    ProfileEffect,
    NameplateEffect,
}

impl ProfileDecorationEquipSlot {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IconFrame => "iconFrame",
            Self::ProfileEffect => "profileEffect",
            Self::NameplateEffect => "nameplateEffect",
        }
    }
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(tag = "assetKind", rename_all = "camelCase", deny_unknown_fields)]
pub enum MediaAssetUploadRequest {
    #[serde(rename_all = "camelCase")]
    Gallery { image_data: String },
    #[serde(rename_all = "camelCase")]
    Icons { image_data: String },
    #[serde(rename_all = "camelCase")]
    Emojis {
        image_data: String,
        params: EmojiUploadParams,
    },
    #[serde(rename_all = "camelCase")]
    Stickers { image_data: String },
    #[serde(rename_all = "camelCase")]
    Prints {
        image_data: String,
        crop_white_border: bool,
        params: PrintUploadParams,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryItemUpdateRequest {
    pub is_archived: bool,
}
