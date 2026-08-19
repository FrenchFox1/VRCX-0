use serde::Deserialize;
use vrcx_0_application_core::vrchat_api::media::{
    EmojiUploadParams, InventoryItemUpdateRequest, InventoryListParams, MediaFileListParams,
    PrintUploadParams, ProfileDecorationEquipSlot,
};

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaFilesInput {
    #[serde(default)]
    pub(crate) params: MediaFileListParams,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaInventoryItemsInput {
    #[serde(default)]
    pub(crate) params: InventoryListParams,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaFileIdInput {
    #[serde(default)]
    pub(crate) file_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaImageUploadInput {
    #[serde(default)]
    pub(crate) image_data: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaEmojiUploadInput {
    #[serde(default)]
    pub(crate) image_data: String,
    pub(crate) params: EmojiUploadParams,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaAvatarGalleryImageUploadInput {
    #[serde(default)]
    pub(crate) image_data: String,
    pub(crate) avatar_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaPrintUploadInput {
    #[serde(default)]
    pub(crate) image_data: String,
    #[serde(default)]
    pub(crate) crop_white_border: bool,
    pub(crate) params: PrintUploadParams,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaPrintsInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) n: i64,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaPrintIdInput {
    #[serde(default)]
    pub(crate) print_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatPrintFavoriteSetInput {
    #[serde(default)]
    pub(crate) print_id: String,
    #[serde(default)]
    pub(crate) favorite: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaUserInventoryItemInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) inventory_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaInventoryItemInput {
    #[serde(default)]
    pub(crate) inventory_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaInventoryItemUpdateInput {
    #[serde(default)]
    pub(crate) inventory_id: String,
    pub(crate) params: InventoryItemUpdateRequest,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaInventoryTemplateInput {
    #[serde(default)]
    pub(crate) inventory_template_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaProfileDecorationEquipInput {
    #[serde(default)]
    pub(crate) inventory_id: String,
    pub(crate) equip_slot: ProfileDecorationEquipSlot,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatMediaProfileDecorationUnequipInput {
    pub(crate) equip_slot: ProfileDecorationEquipSlot,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaRewardRedeemInput {
    #[serde(default)]
    pub(crate) code: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VrchatMediaLegacyImageUploadInput {
    #[serde(default)]
    pub(crate) entity_id: String,
    #[serde(default)]
    pub(crate) image_url: String,
    #[serde(default)]
    pub(crate) base64_file: String,
    #[serde(default)]
    pub(crate) file_size_in_bytes: Option<i64>,
}
