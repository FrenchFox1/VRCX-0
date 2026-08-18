use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::query::serialize_query;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarListParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

impl CalendarListParams {
    pub(crate) fn into_query_params(self) -> HashMap<String, Value> {
        serialize_query(&self)
    }
}
