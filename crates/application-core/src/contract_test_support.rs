use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::Mutex;

use serde::Serialize;

use crate::{Error, ImageCachePort, Result, WebClient, WebClientPort, WorldCachePort};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BehaviorLockFacet {
    InputNormalization,
    OutputAndSerialization,
    ErrorMapping,
    AuthenticationAndOwnership,
    RemoteRequest,
    Persistence,
    Events,
    Cache,
    Lifecycle,
    Diagnostics,
    RawBoundary,
}

pub const BEHAVIOR_LOCK_CHECKLIST: &[BehaviorLockFacet] = &[
    BehaviorLockFacet::InputNormalization,
    BehaviorLockFacet::OutputAndSerialization,
    BehaviorLockFacet::ErrorMapping,
    BehaviorLockFacet::AuthenticationAndOwnership,
    BehaviorLockFacet::RemoteRequest,
    BehaviorLockFacet::Persistence,
    BehaviorLockFacet::Events,
    BehaviorLockFacet::Cache,
    BehaviorLockFacet::Lifecycle,
    BehaviorLockFacet::Diagnostics,
    BehaviorLockFacet::RawBoundary,
];

pub struct CallRecorder<T> {
    calls: Mutex<Vec<T>>,
}

impl<T> Default for CallRecorder<T> {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl<T> CallRecorder<T> {
    pub fn record(&self, call: T) {
        self.calls.lock().expect("call recorder lock").push(call);
    }

    pub fn len(&self) -> usize {
        self.calls.lock().expect("call recorder lock").len()
    }

    pub fn is_empty(&self) -> bool {
        self.calls.lock().expect("call recorder lock").is_empty()
    }
}

impl<T: Clone> CallRecorder<T> {
    pub fn snapshot(&self) -> Vec<T> {
        self.calls.lock().expect("call recorder lock").clone()
    }
}

pub struct ScriptedResults<T> {
    results: Mutex<VecDeque<T>>,
}

impl<T> ScriptedResults<T> {
    pub fn new(results: impl IntoIterator<Item = T>) -> Self {
        Self {
            results: Mutex::new(results.into_iter().collect()),
        }
    }

    pub fn next(&self) -> T {
        self.results
            .lock()
            .expect("scripted result lock")
            .pop_front()
            .expect("scripted result exhausted")
    }
}

pub fn assert_json_contract<T: Serialize>(actual: &T, expected: serde_json::Value) {
    assert_eq!(
        serde_json::to_value(actual).expect("serialize contract value"),
        expected
    );
}

#[derive(Default)]
pub struct NoopWebClientPort;

#[async_trait::async_trait]
impl WebClientPort for NoopWebClientPort {
    fn save_cookies(&self) {}
    fn proxy_url(&self) -> Option<String> {
        None
    }
    async fn fetch_image(&self, _url: &str) -> Result<Vec<u8>> {
        Err(Error::Custom("noop web client".into()))
    }
    fn realtime_connection_options(&self) -> vrcx_0_contracts::RealtimeConnectionOptions {
        vrcx_0_contracts::RealtimeConnectionOptions {
            origin: String::new(),
            proxy_url: None,
        }
    }
    fn clear_cookies(&self) {}
    fn clear_auth_cookies(&self) {}
    fn cookie_diagnostics(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn auth_cookie_value(&self) -> Option<String> {
        None
    }
    fn get_cookies(&self) -> String {
        String::new()
    }
    fn set_cookies(&self, _b64: &str) -> Result<()> {
        Ok(())
    }
    async fn execute(
        &self,
        _request: vrcx_0_contracts::WebExecuteRequest,
    ) -> Result<(i32, String)> {
        Err(Error::Custom("noop web client".into()))
    }
    async fn execute_external(
        &self,
        _request: vrcx_0_contracts::external_api::ExternalWebExecuteRequest,
    ) -> Result<(i32, String)> {
        Err(Error::Custom("noop web client".into()))
    }
    async fn execute_api(
        &self,
        _input: vrcx_0_contracts::VrchatRequest,
        _scope: vrcx_0_contracts::VrchatScope,
    ) -> Result<vrcx_0_contracts::VrchatResponse> {
        Err(Error::Custom("noop web client".into()))
    }
    fn vrchat_config_snapshot(&self, _endpoint: &str) -> Option<vrcx_0_contracts::VrchatResponse> {
        None
    }
    fn clear_vrchat_config_snapshot(&self) {}
    async fn fetch_realtime_auth_token(
        &self,
        _endpoint: &str,
    ) -> Result<vrcx_0_contracts::RealtimeAuthTokenFetch> {
        Err(Error::Custom("noop web client".into()))
    }
    async fn execute_external_api(
        &self,
        _input: vrcx_0_contracts::external_api::ExternalHttpRequestInput,
        _scope: vrcx_0_contracts::external_api::ExternalApiScope,
    ) -> Result<vrcx_0_contracts::external_api::ExternalApiExecuteResponse> {
        Err(Error::Custom("noop web client".into()))
    }
    async fn execute_external_api_limited(
        &self,
        _input: vrcx_0_contracts::external_api::ExternalHttpRequestInput,
        _scope: vrcx_0_contracts::external_api::ExternalApiScope,
        _max_response_bytes: usize,
    ) -> Result<vrcx_0_contracts::external_api::ExternalApiExecuteResponse> {
        Err(Error::Custom("noop web client".into()))
    }
}

#[derive(Default)]
pub struct MemoryCookieWebClientPort {
    cookies: Mutex<String>,
}

#[async_trait::async_trait]
impl WebClientPort for MemoryCookieWebClientPort {
    fn save_cookies(&self) {}
    fn proxy_url(&self) -> Option<String> {
        None
    }
    async fn fetch_image(&self, _url: &str) -> Result<Vec<u8>> {
        Err(Error::Custom("memory cookie web client".into()))
    }
    fn realtime_connection_options(&self) -> vrcx_0_contracts::RealtimeConnectionOptions {
        vrcx_0_contracts::RealtimeConnectionOptions {
            origin: String::new(),
            proxy_url: None,
        }
    }
    fn clear_cookies(&self) {
        self.cookies.lock().expect("memory cookie lock").clear();
    }
    fn clear_auth_cookies(&self) {
        self.clear_cookies();
    }
    fn cookie_diagnostics(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
    fn auth_cookie_value(&self) -> Option<String> {
        None
    }
    fn get_cookies(&self) -> String {
        self.cookies.lock().expect("memory cookie lock").clone()
    }
    fn set_cookies(&self, b64: &str) -> Result<()> {
        *self.cookies.lock().expect("memory cookie lock") = b64.to_string();
        Ok(())
    }
    async fn execute(
        &self,
        _request: vrcx_0_contracts::WebExecuteRequest,
    ) -> Result<(i32, String)> {
        Err(Error::Custom("memory cookie web client".into()))
    }
    async fn execute_external(
        &self,
        _request: vrcx_0_contracts::external_api::ExternalWebExecuteRequest,
    ) -> Result<(i32, String)> {
        Err(Error::Custom("memory cookie web client".into()))
    }
    async fn execute_api(
        &self,
        _input: vrcx_0_contracts::VrchatRequest,
        _scope: vrcx_0_contracts::VrchatScope,
    ) -> Result<vrcx_0_contracts::VrchatResponse> {
        Err(Error::Custom("memory cookie web client".into()))
    }
    fn vrchat_config_snapshot(&self, _endpoint: &str) -> Option<vrcx_0_contracts::VrchatResponse> {
        None
    }
    fn clear_vrchat_config_snapshot(&self) {}
    async fn fetch_realtime_auth_token(
        &self,
        _endpoint: &str,
    ) -> Result<vrcx_0_contracts::RealtimeAuthTokenFetch> {
        Err(Error::Custom("memory cookie web client".into()))
    }
    async fn execute_external_api(
        &self,
        _input: vrcx_0_contracts::external_api::ExternalHttpRequestInput,
        _scope: vrcx_0_contracts::external_api::ExternalApiScope,
    ) -> Result<vrcx_0_contracts::external_api::ExternalApiExecuteResponse> {
        Err(Error::Custom("memory cookie web client".into()))
    }
    async fn execute_external_api_limited(
        &self,
        _input: vrcx_0_contracts::external_api::ExternalHttpRequestInput,
        _scope: vrcx_0_contracts::external_api::ExternalApiScope,
        _max_response_bytes: usize,
    ) -> Result<vrcx_0_contracts::external_api::ExternalApiExecuteResponse> {
        Err(Error::Custom("memory cookie web client".into()))
    }
}

#[derive(Default)]
pub struct NoopImageCachePort;

#[async_trait::async_trait]
impl ImageCachePort for NoopImageCachePort {
    async fn get_image(&self, _url: &str, _file_id: &str, _version: &str) -> Result<String> {
        Err(Error::Custom("noop image cache".into()))
    }
    async fn save_image_to_file(&self, _url: &str, _path: &str) -> Result<()> {
        Err(Error::Custom("noop image cache".into()))
    }
    async fn save_ugc_image_to_file(
        &self,
        _url: &str,
        _ugc_folder_path: &str,
        _category: vrcx_0_contracts::UgcCategory,
        _month_folder: &str,
        _file_name: &str,
    ) -> Result<String> {
        Err(Error::Custom("noop image cache".into()))
    }
}

#[derive(Default)]
pub struct NoopWorldCachePort;

#[async_trait::async_trait]
impl WorldCachePort for NoopWorldCachePort {
    fn clear_working(&self) {}
    fn get_name(&self, _world_id: &str) -> Option<String> {
        None
    }
    fn get_summary(&self, _world_id: &str) -> Result<Option<vrcx_0_contracts::WorldSummaryOutput>> {
        Ok(None)
    }
    fn get_cached_card_payload(&self, _world_id: &str) -> Option<serde_json::Value> {
        None
    }
    fn search_summaries(
        &self,
        _query: &str,
        _limit: i64,
    ) -> Result<Vec<vrcx_0_contracts::WorldSummaryOutput>> {
        Ok(Vec::new())
    }
    fn hydrate_from_payload(&self, _world_value: &serde_json::Value) -> Option<String> {
        None
    }
    fn hydrate_summary_from_payload(
        &self,
        _world_value: &serde_json::Value,
    ) -> Option<vrcx_0_contracts::WorldSummaryOutput> {
        None
    }
    fn hydrate_favorite_payloads(
        &self,
        world_values: &[serde_json::Value],
    ) -> Vec<Option<serde_json::Value>> {
        vec![None; world_values.len()]
    }
    async fn resolve_name(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        _world_id: &str,
    ) -> Option<String> {
        None
    }
    async fn resolve_summary(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        _world_id: &str,
    ) -> Option<vrcx_0_contracts::WorldSummaryOutput> {
        None
    }
    async fn resolve_image_url(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        _world_id: &str,
    ) -> Option<String> {
        None
    }
    async fn get(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        _world_id: &str,
        _force: bool,
        _full: bool,
    ) -> Result<vrcx_0_contracts::VrchatResponse> {
        Err(Error::Custom("noop world cache".into()))
    }
    fn hydrate_response(&self, _response: &vrcx_0_contracts::VrchatResponse) {}
}

#[derive(Clone, Default)]
pub struct MemoryWorldCachePort {
    worlds: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl MemoryWorldCachePort {
    pub fn insert(&self, world: serde_json::Value) {
        let Some(id) = world.get("id").and_then(serde_json::Value::as_str) else {
            return;
        };
        self.worlds
            .lock()
            .expect("memory world cache lock")
            .insert(id.to_string(), world);
    }

    fn world(&self, world_id: &str) -> Option<serde_json::Value> {
        self.worlds
            .lock()
            .expect("memory world cache lock")
            .get(world_id)
            .cloned()
    }

    fn summary(&self, world_id: &str) -> Option<vrcx_0_contracts::WorldSummaryOutput> {
        let world = self.world(world_id)?;
        let string = |key: &str| {
            world
                .get(key)
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        Some(vrcx_0_contracts::WorldSummaryOutput {
            id: string("id"),
            author_id: string("authorId"),
            author_name: string("authorName"),
            created_at: string("createdAt").into(),
            description: string("description"),
            image_url: string("imageUrl"),
            name: string("name"),
            release_status: string("releaseStatus").into(),
            thumbnail_image_url: string("thumbnailImageUrl"),
            updated_at: string("updatedAt").into(),
            version: world
                .get("version")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or_default(),
        })
    }
}

#[async_trait::async_trait]
impl WorldCachePort for MemoryWorldCachePort {
    fn clear_working(&self) {}
    fn get_name(&self, world_id: &str) -> Option<String> {
        self.world(world_id)?
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
    }
    fn get_summary(&self, world_id: &str) -> Result<Option<vrcx_0_contracts::WorldSummaryOutput>> {
        Ok(self.summary(world_id))
    }
    fn get_cached_card_payload(&self, world_id: &str) -> Option<serde_json::Value> {
        self.world(world_id)
    }
    fn search_summaries(
        &self,
        _query: &str,
        _limit: i64,
    ) -> Result<Vec<vrcx_0_contracts::WorldSummaryOutput>> {
        Ok(Vec::new())
    }
    fn hydrate_from_payload(&self, world_value: &serde_json::Value) -> Option<String> {
        self.insert(world_value.clone());
        world_value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
    }
    fn hydrate_summary_from_payload(
        &self,
        world_value: &serde_json::Value,
    ) -> Option<vrcx_0_contracts::WorldSummaryOutput> {
        self.insert(world_value.clone());
        let world_id = world_value.get("id").and_then(serde_json::Value::as_str)?;
        self.summary(world_id)
    }
    fn hydrate_favorite_payloads(
        &self,
        world_values: &[serde_json::Value],
    ) -> Vec<Option<serde_json::Value>> {
        world_values
            .iter()
            .map(|world| {
                let mut card = world.clone();
                if let Some(card) = card.as_object_mut() {
                    card.remove("unityPackages");
                    card.remove("instances");
                }
                self.insert(card.clone());
                Some(card)
            })
            .collect()
    }
    async fn resolve_name(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        world_id: &str,
    ) -> Option<String> {
        self.get_name(world_id)
    }
    async fn resolve_summary(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        world_id: &str,
    ) -> Option<vrcx_0_contracts::WorldSummaryOutput> {
        self.summary(world_id)
    }
    async fn resolve_image_url(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        world_id: &str,
    ) -> Option<String> {
        self.world(world_id)?
            .get("imageUrl")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
    }
    async fn get(
        &self,
        _web: &WebClient,
        _endpoint: &str,
        world_id: &str,
        _force: bool,
        _full: bool,
    ) -> Result<vrcx_0_contracts::VrchatResponse> {
        let world = self
            .world(world_id)
            .ok_or_else(|| Error::Custom("memory world cache miss".into()))?;
        Ok(vrcx_0_contracts::VrchatResponse {
            status: 200,
            data: world.to_string(),
        })
    }
    fn hydrate_response(&self, response: &vrcx_0_contracts::VrchatResponse) {
        if let Ok(world) = serde_json::from_str(&response.data) {
            self.insert(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checklist_has_every_observable_behavior_category_once() {
        assert_eq!(BEHAVIOR_LOCK_CHECKLIST.len(), 11);
        for (index, facet) in BEHAVIOR_LOCK_CHECKLIST.iter().enumerate() {
            assert!(!BEHAVIOR_LOCK_CHECKLIST[..index].contains(facet));
        }
    }

    #[test]
    fn recorder_and_scripted_results_are_deterministic() {
        let calls = CallRecorder::default();
        calls.record("first");
        calls.record("second");
        assert_eq!(calls.snapshot(), ["first", "second"]);

        let results = ScriptedResults::new([1, 2]);
        assert_eq!(results.next(), 1);
        assert_eq!(results.next(), 2);
    }
}
