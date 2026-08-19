use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::query::deserialize_optional_nonnegative_i32"
    )]
    pub n: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::query::deserialize_optional_nonnegative_i32"
    )]
    pub offset: Option<i32>,
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
pub enum ImageMaskTag {
    Square,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum EmojiLoopStyle {
    PingPong,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(tag = "tag", rename_all = "lowercase", deny_unknown_fields)]
pub enum EmojiUploadParams {
    #[serde(rename_all = "camelCase")]
    Emoji {
        animation_style: ImageAnimationStyle,
        mask_tag: ImageMaskTag,
    },
    #[serde(rename_all = "camelCase")]
    EmojiAnimated {
        animation_style: ImageAnimationStyle,
        mask_tag: ImageMaskTag,
        #[serde(deserialize_with = "deserialize_emoji_frame_count")]
        frames: i32,
        #[serde(deserialize_with = "deserialize_emoji_frames_per_second")]
        frames_over_time: i32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        loop_style: Option<EmojiLoopStyle>,
    },
}

fn deserialize_bounded_i32<'de, D>(
    deserializer: D,
    minimum: i32,
    maximum: i32,
    field: &str,
) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i32::deserialize(deserializer)?;
    if (minimum..=maximum).contains(&value) {
        Ok(value)
    } else {
        Err(D::Error::custom(format!(
            "{field} must be between {minimum} and {maximum}"
        )))
    }
}

fn deserialize_emoji_frame_count<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_i32(deserializer, 2, 64, "frames")
}

fn deserialize_emoji_frames_per_second<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_i32(deserializer, 1, 64, "framesOverTime")
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::query::deserialize_optional_nonnegative_i32"
    )]
    pub n: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::query::deserialize_optional_nonnegative_i32"
    )]
    pub offset: Option<i32>,
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
