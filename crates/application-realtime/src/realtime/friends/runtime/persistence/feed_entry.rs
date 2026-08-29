use serde_json::Value;
use vrcx_0_core::json::text_of;

pub(super) fn feed_duration_ms(duration_ms: i64) -> Option<i64> {
    (duration_ms > 0).then_some(duration_ms)
}

pub(super) fn feed_avatar_tags(value: Option<&Value>) -> Option<Vec<String>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    Some(
        value
            .as_array()?
            .iter()
            .filter(|item| !item.is_null())
            .map(|item| text_of(Some(item)))
            .collect(),
    )
}
