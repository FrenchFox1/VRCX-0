use serde::{Deserialize, Serialize};

use crate::query::{
    QueryOrder, ReleaseStatusFilter, UserSearchCustomField, UserSearchSort, WorldSearchSort,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldSearchParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<WorldSearchSort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<QueryOrder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_status: Option<ReleaseStatusFilter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_unity_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_unity_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noplatform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fuzzy: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_specific: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UserSearchParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_internal_variant: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<UserSearchCustomField>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<UserSearchSort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<QueryOrder>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GroupSearchParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
}
