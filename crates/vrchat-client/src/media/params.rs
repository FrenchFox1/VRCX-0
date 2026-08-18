use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum MediaFileTag {
    Gallery,
    AvatarGallery,
    Icon,
    Emoji,
    EmojiAnimated,
    Sticker,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaFileListParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<MediaFileTag>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum ImageAnimationStyle {
    Aura,
    Bats,
    Bees,
    Bounce,
    Cloud,
    Confetti,
    Crying,
    Dislike,
    Fire,
    Idea,
    Lasers,
    Like,
    Magnet,
    Mistletoe,
    Money,
    Noise,
    Orbit,
    Pizza,
    Rain,
    Rotate,
    Shake,
    Snow,
    Snowball,
    Spin,
    Splash,
    Stop,
    Zzz,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum EmojiFileTag {
    Emoji,
    EmojiAnimated,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum ImageMaskTag {
    Square,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum EmojiLoopStyle {
    PingPong,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EmojiUploadParams {
    pub tag: EmojiFileTag,
    pub animation_style: ImageAnimationStyle,
    pub mask_tag: ImageMaskTag,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames_over_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_style: Option<EmojiLoopStyle>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrintUploadParams {
    pub note: String,
    pub timestamp: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum InventoryOrder {
    Newest,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryListParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equip_slot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<InventoryOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_types: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_flags: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
}
