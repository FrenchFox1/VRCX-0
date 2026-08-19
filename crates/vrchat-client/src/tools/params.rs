use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::query::serialize_query;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarListParams {
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
    pub date: Option<String>,
}

impl CalendarListParams {
    pub(crate) fn into_query_params(self) -> HashMap<String, Value> {
        serialize_query(&self)
    }
}
