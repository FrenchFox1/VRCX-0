use std::future::Future;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use vrcx_0_core::json::RawJson;
use vrcx_0_persistence::DatabaseService;
use vrcx_0_vrchat_client::{
    http_api::{ApiJsonResponse, ApiScope},
    media::{inventory_items_get_input, InventoryListParams, InventoryOrder},
};

use vrcx_0_application_core::{
    Error, Result, RuntimeAuthScope, RuntimeAuthScopeSnapshot, WebClient,
};

const INVENTORY_COLLECT_PAGE_SIZE: usize = 100;
const INVENTORY_COLLECT_MAX_PAGES: usize = 100;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum InventoryQueryItemType {
    Bundle,
    DroneSkin,
    Emoji,
    #[serde(rename = "iconFrame")]
    IconFrame,
    #[serde(rename = "nameplateEffect")]
    NameplateEffect,
    PortalSkin,
    #[serde(rename = "profileEffect")]
    ProfileEffect,
    Prop,
    Sticker,
    WarpEffect,
}

impl InventoryQueryItemType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Bundle => "bundle",
            Self::DroneSkin => "droneskin",
            Self::Emoji => "emoji",
            Self::IconFrame => "iconFrame",
            Self::NameplateEffect => "nameplateEffect",
            Self::PortalSkin => "portalskin",
            Self::ProfileEffect => "profileEffect",
            Self::Prop => "prop",
            Self::Sticker => "sticker",
            Self::WarpEffect => "warpeffect",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum InventoryQueryFlag {
    Archivable,
    Cloneable,
    Consumable,
    Equippable,
    Instantiatable,
    Trashable,
    Ugc,
    Unique,
}

impl InventoryQueryFlag {
    fn as_str(self) -> &'static str {
        match self {
            Self::Archivable => "archivable",
            Self::Cloneable => "cloneable",
            Self::Consumable => "consumable",
            Self::Equippable => "equippable",
            Self::Instantiatable => "instantiatable",
            Self::Trashable => "trashable",
            Self::Ugc => "ugc",
            Self::Unique => "unique",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryItemsCollectInput {
    #[serde(default)]
    pub types: Vec<InventoryQueryItemType>,
    #[serde(default)]
    pub not_flags: Vec<InventoryQueryFlag>,
    #[serde(default)]
    pub archived: Option<bool>,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemsCollectOutput {
    pub items: Vec<RawJson>,
    pub truncated: bool,
}

pub struct InventoryItemsCollectDeps<'a> {
    pub db: &'a DatabaseService,
    pub web: &'a WebClient,
    pub auth_scope: &'a RuntimeAuthScope,
    pub expected_scope: RuntimeAuthScopeSnapshot,
}

pub async fn collect_inventory_items(
    deps: &InventoryItemsCollectDeps<'_>,
    input: InventoryItemsCollectInput,
) -> Result<InventoryItemsCollectOutput> {
    let (rows, truncated) =
        collect_paged(|page_index| fetch_inventory_page(deps, &input, page_index)).await?;
    Ok(InventoryItemsCollectOutput {
        items: rows.into_iter().map(RawJson::from).collect(),
        truncated,
    })
}

async fn collect_paged<F, Fut>(mut fetch_page: F) -> Result<(Vec<Value>, bool)>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<Vec<Value>>>,
{
    let mut rows = Vec::new();
    for page_index in 0..INVENTORY_COLLECT_MAX_PAGES {
        let page = fetch_page(page_index).await?;
        let page_len = page.len();
        rows.extend(page);
        if page_len < INVENTORY_COLLECT_PAGE_SIZE {
            return Ok((rows, false));
        }
    }
    Ok((rows, true))
}

async fn fetch_inventory_page(
    deps: &InventoryItemsCollectDeps<'_>,
    input: &InventoryItemsCollectInput,
    page_index: usize,
) -> Result<Vec<Value>> {
    ensure_scope_matches(&deps.auth_scope.snapshot(), &deps.expected_scope)?;
    let request = inventory_items_get_input(
        deps.expected_scope.endpoint.clone(),
        page_request_params(input, page_index),
    );
    let response = deps
        .web
        .execute_api(request, ApiScope::VrchatMedia, deps.db)
        .await?;
    ensure_scope_matches(&deps.auth_scope.snapshot(), &deps.expected_scope)?;
    parse_inventory_page(ApiJsonResponse::parse(response.status, &response.data))
}

fn page_request_params(
    input: &InventoryItemsCollectInput,
    page_index: usize,
) -> InventoryListParams {
    let types = input
        .types
        .iter()
        .map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let not_flags = input
        .not_flags
        .iter()
        .map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join(",");
    InventoryListParams {
        n: Some(INVENTORY_COLLECT_PAGE_SIZE as i32),
        offset: Some((page_index * INVENTORY_COLLECT_PAGE_SIZE) as i32),
        order: Some(InventoryOrder::Newest),
        types: (!types.is_empty()).then_some(types),
        not_flags: (!not_flags.is_empty()).then_some(not_flags),
        archived: input.archived,
        ..Default::default()
    }
}

fn parse_inventory_page(response: ApiJsonResponse) -> Result<Vec<Value>> {
    if response.is_failure() {
        return Err(Error::Custom(
            response.error_message_or("VRChat inventory collect failed"),
        ));
    }
    Ok(response
        .json
        .get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

fn ensure_scope_matches(
    current: &RuntimeAuthScopeSnapshot,
    expected: &RuntimeAuthScopeSnapshot,
) -> Result<()> {
    if current.generation_matches(expected) {
        Ok(())
    } else {
        Err(Error::Custom(
            "Inventory collect authentication scope changed.".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn full_page() -> Vec<Value> {
        (0..INVENTORY_COLLECT_PAGE_SIZE)
            .map(|index| json!({ "id": format!("inv_{index}") }))
            .collect()
    }

    #[tokio::test]
    async fn collect_paged_stops_on_short_page() {
        let mut pages = vec![full_page(), vec![json!({ "id": "inv_last" })]].into_iter();
        let mut requested_pages = Vec::new();
        let (rows, truncated) = collect_paged(|page_index| {
            requested_pages.push(page_index);
            let page = pages.next().unwrap();
            async move { Ok(page) }
        })
        .await
        .unwrap();

        assert_eq!(requested_pages, vec![0, 1]);
        assert_eq!(rows.len(), INVENTORY_COLLECT_PAGE_SIZE + 1);
        assert!(!truncated);
    }

    #[tokio::test]
    async fn collect_paged_stops_on_empty_first_page() {
        let (rows, truncated) = collect_paged(|_| async { Ok(Vec::new()) }).await.unwrap();
        assert!(rows.is_empty());
        assert!(!truncated);
    }

    #[tokio::test]
    async fn collect_paged_marks_truncation_at_page_limit() {
        let (rows, truncated) = collect_paged(|_| async { Ok(full_page()) }).await.unwrap();
        assert_eq!(
            rows.len(),
            INVENTORY_COLLECT_PAGE_SIZE * INVENTORY_COLLECT_MAX_PAGES
        );
        assert!(truncated);
    }

    #[tokio::test]
    async fn collect_paged_propagates_page_errors() {
        let error = collect_paged(|_| async { Err::<Vec<Value>, _>(Error::Custom("boom".into())) })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("boom"));
    }

    #[test]
    fn page_request_params_serialize_typed_filters_and_own_pagination() {
        let input = InventoryItemsCollectInput {
            types: vec![
                InventoryQueryItemType::IconFrame,
                InventoryQueryItemType::ProfileEffect,
            ],
            not_flags: vec![InventoryQueryFlag::Ugc],
            archived: Some(false),
        };

        let params = page_request_params(&input, 3);

        assert_eq!(
            params.order,
            Some(vrcx_0_vrchat_client::media::InventoryOrder::Newest)
        );
        assert_eq!(params.n, Some(100));
        assert_eq!(params.offset, Some(300));
        assert_eq!(params.types.as_deref(), Some("iconFrame,profileEffect"));
        assert_eq!(params.not_flags.as_deref(), Some("ugc"));
        assert_eq!(params.archived, Some(false));
    }

    #[test]
    fn collect_input_rejects_unknown_filters_and_fields() {
        assert!(serde_json::from_value::<InventoryItemsCollectInput>(json!({
            "types": ["futureType"],
        }))
        .is_err());
        assert!(serde_json::from_value::<InventoryItemsCollectInput>(json!({
            "flags": ["ugc"],
        }))
        .is_err());
    }

    fn page_response(status: i32, json: Value) -> ApiJsonResponse {
        ApiJsonResponse { status, json }
    }

    #[test]
    fn parse_inventory_page_extracts_data_rows() {
        let rows = parse_inventory_page(page_response(
            200,
            json!({ "data": [{ "id": "inv_1" }], "totalCount": 1 }),
        ))
        .unwrap();
        assert_eq!(rows, vec![json!({ "id": "inv_1" })]);

        assert!(
            parse_inventory_page(page_response(200, json!({ "totalCount": 0 })))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn parse_inventory_page_rejects_error_responses() {
        let error = parse_inventory_page(page_response(
            403,
            json!({ "error": { "message": "denied" } }),
        ))
        .unwrap_err();
        assert!(error.to_string().contains("denied"));

        let error = parse_inventory_page(page_response(
            200,
            json!({ "error": { "message": "soft error" } }),
        ))
        .unwrap_err();
        assert!(error.to_string().contains("soft error"));

        let error = parse_inventory_page(page_response(500, json!({}))).unwrap_err();
        assert!(error.to_string().contains("500"));
    }

    #[test]
    fn scope_mismatch_is_rejected() {
        let expected = RuntimeAuthScopeSnapshot {
            current_user_id: "usr_self".into(),
            endpoint: "https://api.vrchat.cloud/api/1".into(),
            generation: 1,
            active: true,
        };
        assert!(ensure_scope_matches(&expected.clone(), &expected).is_ok());

        let mut stale = expected.clone();
        stale.generation = 2;
        assert!(ensure_scope_matches(&stale, &expected).is_err());

        let mut inactive = expected.clone();
        inactive.active = false;
        assert!(ensure_scope_matches(&inactive, &expected).is_err());
    }
}
