use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldUpdateRequest {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::query::deserialize_optional_nonnegative_i32"
    )]
    pub capacity: Option<i32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::query::deserialize_optional_nonnegative_i32"
    )]
    pub recommended_capacity: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_youtube_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_list: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_prop_abilities: Option<Vec<String>>,
}
