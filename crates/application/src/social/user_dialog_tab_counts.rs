use futures_util::future::BoxFuture;

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use std::future::Future;

use futures_util::stream::{self, StreamExt};
use moka::future::Cache;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use vrcx_0_application_core::{
    vrchat_api::{VrchatApiRequest, VrchatScope},
    RuntimeAuthScope, RuntimeAuthScopeSnapshot, WebClient,
};
use vrcx_0_contracts::VrchatJsonResponse;

use vrcx_0_application_core::{Error, Result};
use vrcx_0_core::json::RawJson;

const MAX_PROFILE_PAGES: usize = 50;
const TAB_COUNTS_CACHE_CAPACITY: u64 = 32;
const TAB_COUNTS_CACHE_TTL: Duration = Duration::from_secs(10 * 60);
const WORLD_PAGE_SIZE: usize = 100;
const WORLD_MAX_OFFSET: i32 = ((MAX_PROFILE_PAGES - 1) * WORLD_PAGE_SIZE) as i32;
const FAVORITE_GROUP_PAGE_SIZE: usize = 50;
const FAVORITE_GROUP_FETCH_CONCURRENCY: usize = 8;
const FAVORITE_WORLD_PAGE_SIZE: usize = 300;
const FAVORITE_WORLD_MAX_OFFSET: i32 = ((MAX_PROFILE_PAGES - 1) * FAVORITE_WORLD_PAGE_SIZE) as i32;
const MY_AVATAR_PAGE_SIZE: usize = 50;
const MY_AVATAR_MAX_OFFSET: i32 = 5_000;
pub const DEFAULT_AVATAR_PROVIDER: &str = "https://api.avtrdb.com/v3/avatar/search/vrcx";

#[derive(Clone)]
pub struct UserDialogTabCountsDeps {
    source: Arc<dyn UserDialogTabCountsSource>,
    pub(crate) web: Arc<WebClient>,
    pub auth_scope: RuntimeAuthScope,
}

impl UserDialogTabCountsDeps {
    pub fn new(
        source: Arc<dyn UserDialogTabCountsSource>,
        web: Arc<WebClient>,
        auth_scope: RuntimeAuthScope,
    ) -> Self {
        Self {
            source,
            web,
            auth_scope,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AvatarProviderConfig {
    pub enabled: bool,
    pub providers: RawJson,
    pub selected: String,
}

pub type UserDialogExternalFuture<'a> =
    BoxFuture<'a, Result<(i32, String)>>;

pub trait UserDialogTabCountsSource: Send + Sync {
    fn avatar_provider_config(&self) -> Result<AvatarProviderConfig>;
    fn mutual_friends(&self, endpoint: String, user_id: String) -> Result<VrchatApiRequest>;
    fn groups(&self, endpoint: String, user_id: String) -> Result<VrchatApiRequest>;
    fn worlds(
        &self,
        endpoint: String,
        user_id: String,
        n: i32,
        offset: i32,
        release_status: AvatarReleaseStatus,
    ) -> Result<VrchatApiRequest>;
    fn favorite_worlds(
        &self,
        endpoint: String,
        user_id: String,
        group_name: String,
        n: i32,
        offset: i32,
    ) -> Result<VrchatApiRequest>;
    fn favorite_groups(
        &self,
        endpoint: String,
        user_id: String,
        n: i32,
        offset: i32,
    ) -> Result<VrchatApiRequest>;
    fn my_avatars(&self, endpoint: String, n: i32, offset: i32) -> Result<VrchatApiRequest>;
    fn external_avatar_search<'a>(
        &'a self,
        provider: &'a str,
        target_user_id: &'a str,
    ) -> UserDialogExternalFuture<'a>;
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
#[specta(rename = "UserDialogAvatarReleaseStatus")]
pub enum AvatarReleaseStatus {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "public")]
    Public,
}

impl AvatarReleaseStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Hidden => "hidden",
            Self::Private => "private",
            Self::Public => "public",
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct UserDialogTabCountsCacheKey {
    auth_generation: u64,
    current_user_id: String,
    endpoint: String,
    target_user_id: String,
    avatar_release_status: String,
    avatar_provider: String,
    include_mutual_friends: bool,
}

#[derive(Clone)]
pub struct UserDialogTabCountsRuntime {
    cache: Cache<UserDialogTabCountsCacheKey, Arc<UserDialogTabCountsOutput>>,
}

impl UserDialogTabCountsRuntime {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(TAB_COUNTS_CACHE_CAPACITY)
                .time_to_live(TAB_COUNTS_CACHE_TTL)
                .build(),
        }
    }

    async fn resolve<F, Fut>(
        &self,
        key: UserDialogTabCountsCacheKey,
        force: bool,
        include_mutual_friends: bool,
        load: F,
    ) -> Result<UserDialogTabCountsOutput>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<UserDialogTabCountsOutput>>,
    {
        if force {
            self.cache.invalidate(&key).await;
        }
        let counts = self
            .cache
            .try_get_with(key.clone(), async move { load().await.map(Arc::new) })
            .await
            .map_err(|error| Error::Custom(error.to_string()))?;
        if !counts.is_complete(include_mutual_friends) {
            self.cache.invalidate(&key).await;
        }
        Ok((*counts).clone())
    }
}

impl Default for UserDialogTabCountsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn get_user_dialog_tab_counts(
    runtime: &UserDialogTabCountsRuntime,
    deps: UserDialogTabCountsDeps,
    input: UserDialogTabCountsInput,
) -> Result<UserDialogTabCountsOutput> {
    let scope = require_active_scope(&deps.auth_scope)?;
    let target_user_id = input.user_id.trim().to_string();
    if target_user_id.is_empty() {
        return Err(Error::Custom(
            "User dialog tab counts require a user id.".into(),
        ));
    }
    let avatar_release_status = input.avatar_release_status;
    let avatar_provider = if target_user_id == scope.current_user_id {
        Ok(None)
    } else {
        selected_avatar_provider(deps.source.as_ref())
    };
    let avatar_provider_key = match &avatar_provider {
        Ok(Some(provider)) => provider.clone(),
        Ok(None) => "disabled".into(),
        Err(_) => "config-error".into(),
    };
    let key = UserDialogTabCountsCacheKey {
        auth_generation: scope.generation,
        current_user_id: scope.current_user_id.clone(),
        endpoint: scope.endpoint.clone(),
        target_user_id: target_user_id.clone(),
        avatar_release_status: avatar_release_status.as_str().to_string(),
        avatar_provider: avatar_provider_key,
        include_mutual_friends: input.include_mutual_friends,
    };
    runtime
        .resolve(
            key,
            input.force,
            input.include_mutual_friends,
            move || async move {
                load_user_dialog_tab_counts(
                    deps,
                    scope,
                    target_user_id,
                    avatar_release_status,
                    avatar_provider,
                    input.include_mutual_friends,
                )
                .await
            },
        )
        .await
}

async fn load_user_dialog_tab_counts(
    deps: UserDialogTabCountsDeps,
    scope: RuntimeAuthScopeSnapshot,
    target_user_id: String,
    avatar_release_status: AvatarReleaseStatus,
    avatar_provider: Result<Option<String>>,
    include_mutual_friends: bool,
) -> Result<UserDialogTabCountsOutput> {
    let mutual_friends = async {
        if include_mutual_friends {
            Some(count_mutual_friends(&deps, &scope, &target_user_id).await)
        } else {
            None
        }
    };
    let avatars = count_avatars(
        &deps,
        &scope,
        &target_user_id,
        avatar_release_status,
        avatar_provider,
    );
    let (mutual_friends, groups, worlds, favorite_worlds, avatars) = tokio::join!(
        mutual_friends,
        count_groups(&deps, &scope, &target_user_id),
        count_worlds(&deps, &scope, &target_user_id),
        count_favorite_worlds(&deps, &scope, &target_user_id),
        avatars,
    );
    Ok(counts_from_results(
        mutual_friends,
        groups,
        worlds,
        favorite_worlds,
        avatars,
    ))
}

async fn count_mutual_friends(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    target_user_id: &str,
) -> Result<usize> {
    let request = deps
        .source
        .mutual_friends(scope.endpoint.clone(), target_user_id.into())?;
    let payload = execute_vrchat_payload(deps, scope, request, "mutual friends").await?;
    let value = serde_json::from_str::<Value>(&payload)?;
    value
        .get("friends")
        .and_then(Value::as_u64)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| Error::Custom("Mutual friend count response is invalid.".into()))
}

async fn count_groups(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    target_user_id: &str,
) -> Result<usize> {
    let request = deps
        .source
        .groups(scope.endpoint.clone(), target_user_id.into())?;
    let payload = execute_vrchat_payload(deps, scope, request, "groups").await?;
    json_array_len(&payload)
}

async fn count_worlds(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    target_user_id: &str,
) -> Result<usize> {
    let release_status = if target_user_id == scope.current_user_id {
        AvatarReleaseStatus::All
    } else {
        AvatarReleaseStatus::Public
    };
    count_payload_pages_bounded(
        WORLD_PAGE_SIZE,
        WORLD_MAX_OFFSET,
        |offset| async move {
            let request = deps.source.worlds(
                scope.endpoint.clone(),
                target_user_id.into(),
                WORLD_PAGE_SIZE as i32,
                offset,
                release_status,
            )?;
            execute_vrchat_payload(deps, scope, request, "worlds").await
        },
        count_all_rows,
    )
    .await
}

async fn count_favorite_worlds(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    target_user_id: &str,
) -> Result<usize> {
    let group_names = collect_world_favorite_group_names(deps, scope, target_user_id).await?;
    let results = stream::iter(group_names)
        .map(|group_name| async move {
            count_payload_pages_bounded(
                FAVORITE_WORLD_PAGE_SIZE,
                FAVORITE_WORLD_MAX_OFFSET,
                |offset| {
                    let group_name = group_name.clone();
                    async move {
                        let request = deps.source.favorite_worlds(
                            scope.endpoint.clone(),
                            target_user_id.into(),
                            group_name,
                            FAVORITE_WORLD_PAGE_SIZE as i32,
                            offset,
                        );
                        execute_vrchat_payload(deps, scope, request?, "favorite worlds").await
                    }
                },
                count_all_rows,
            )
            .await
        })
        .buffer_unordered(FAVORITE_GROUP_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    let mut count = 0;
    for result in results {
        match result {
            Ok(group_count) => count += group_count,
            Err(error) => tracing::debug!(%error, "favorite world count source failed"),
        }
    }
    Ok(count)
}

async fn collect_world_favorite_group_names(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    target_user_id: &str,
) -> Result<Vec<String>> {
    let mut group_names = Vec::new();
    for page in 0..MAX_PROFILE_PAGES {
        let request = deps.source.favorite_groups(
            scope.endpoint.clone(),
            target_user_id.into(),
            FAVORITE_GROUP_PAGE_SIZE as i32,
            (page * FAVORITE_GROUP_PAGE_SIZE) as i32,
        )?;
        let payload = execute_vrchat_payload(deps, scope, request, "favorite groups").await?;
        let page_len = json_array_len(&payload)?;
        group_names.extend(world_favorite_group_names(&payload)?);
        if page_len < FAVORITE_GROUP_PAGE_SIZE || page + 1 == MAX_PROFILE_PAGES {
            return Ok(group_names);
        }
    }
    Ok(group_names)
}

async fn count_avatars(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    target_user_id: &str,
    release_status: AvatarReleaseStatus,
    avatar_provider: Result<Option<String>>,
) -> Result<usize> {
    if target_user_id == scope.current_user_id {
        return count_payload_pages_bounded(
            MY_AVATAR_PAGE_SIZE,
            MY_AVATAR_MAX_OFFSET,
            |offset| async move {
                let request = deps.source.my_avatars(
                    scope.endpoint.clone(),
                    MY_AVATAR_PAGE_SIZE as i32,
                    offset,
                )?;
                execute_vrchat_payload(deps, scope, request, "my avatars").await
            },
            |payload| {
                let page_len = json_array_len(payload)?;
                Ok((page_len, count_my_avatars(payload, release_status)?))
            },
        )
        .await;
    }

    let Some(provider) = avatar_provider? else {
        return Ok(0);
    };
    let (status, payload) = deps
        .source
        .external_avatar_search(&provider, target_user_id)
        .await?;
    if status != 200 {
        return Err(Error::Custom(format!(
            "Avatar search count request failed with status {status}."
        )));
    }
    count_target_avatars(&payload, target_user_id)
}

async fn execute_vrchat_payload(
    deps: &UserDialogTabCountsDeps,
    scope: &RuntimeAuthScopeSnapshot,
    request: VrchatApiRequest,
    source: &str,
) -> Result<String> {
    ensure_scope_matches(&deps.auth_scope, scope)?;
    let response = deps.web.execute_api(request, VrchatScope::Vrchat).await?;
    ensure_scope_matches(&deps.auth_scope, scope)?;
    if response.status >= 400 || response.data.trim_start().starts_with('{') {
        let parsed = VrchatJsonResponse::parse(response.status, &response.data);
        if parsed.is_failure() {
            return Err(Error::Custom(format!(
                "User dialog {source} count request failed: {}",
                parsed.error_message_or("VRChat API request failed")
            )));
        }
    }
    Ok(response.data)
}

fn require_active_scope(auth_scope: &RuntimeAuthScope) -> Result<RuntimeAuthScopeSnapshot> {
    crate::scope_gate::require_active_scope(auth_scope, "User dialog tab counts")
}

fn ensure_scope_matches(
    auth_scope: &RuntimeAuthScope,
    expected: &RuntimeAuthScopeSnapshot,
) -> Result<()> {
    crate::scope_gate::ensure_scope_matches(auth_scope, expected, "User dialog tab counts")
}

fn selected_avatar_provider(source: &dyn UserDialogTabCountsSource) -> Result<Option<String>> {
    let config = source.avatar_provider_config()?;
    if !config.enabled {
        return Ok(None);
    }
    let mut provider_values = match config.providers.into_value() {
        Value::Array(values) => values,
        _ => vec![Value::String(DEFAULT_AVATAR_PROVIDER.into())],
    };
    let selected = config.selected.trim().to_string();
    if !selected.is_empty()
        && !provider_values
            .iter()
            .any(|value| value.as_str() == Some(selected.as_str()))
    {
        provider_values.push(Value::String(selected.clone()));
    }

    let mut seen = HashSet::new();
    let providers = provider_values
        .into_iter()
        .filter_map(|value| value.as_str().and_then(normalize_avatar_provider))
        .filter(|provider| seen.insert(provider.clone()))
        .collect::<Vec<_>>();
    let selected = normalize_avatar_provider(&selected)
        .filter(|provider| providers.contains(provider))
        .or_else(|| providers.first().cloned());
    Ok(selected)
}

fn normalize_avatar_provider(value: &str) -> Option<String> {
    match value.trim() {
        "" | "https://avtr.just-h.party/vrcx_search.php" => None,
        "https://api.avtrdb.com/v1/avatar/search/vrcx"
        | "https://api.avtrdb.com/v2/avatar/search/vrcx" => Some(DEFAULT_AVATAR_PROVIDER.into()),
        value => Some(value.to_string()),
    }
}

fn json_array_len(payload: &str) -> Result<usize> {
    struct ArrayLenVisitor;

    impl<'de> Visitor<'de> for ArrayLenVisitor {
        type Value = usize;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a JSON array")
        }

        fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut count = 0;
            while sequence.next_element::<IgnoredAny>()?.is_some() {
                count += 1;
            }
            Ok(count)
        }
    }

    let mut deserializer = serde_json::Deserializer::from_str(payload);
    Ok(deserializer.deserialize_seq(ArrayLenVisitor)?)
}

fn count_all_rows(payload: &str) -> Result<(usize, usize)> {
    let page_len = json_array_len(payload)?;
    Ok((page_len, page_len))
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UserDialogTabCountsInput {
    pub user_id: String,
    #[serde(default = "default_avatar_release_status")]
    pub avatar_release_status: AvatarReleaseStatus,
    #[serde(default)]
    pub include_mutual_friends: bool,
    #[serde(default)]
    pub force: bool,
}

fn default_avatar_release_status() -> AvatarReleaseStatus {
    AvatarReleaseStatus::All
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct UserDialogTabCountsOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutual_friends: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worlds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorite_worlds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatars: Option<u32>,
}

impl UserDialogTabCountsOutput {
    fn is_complete(&self, include_mutual_friends: bool) -> bool {
        (!include_mutual_friends || self.mutual_friends.is_some())
            && self.groups.is_some()
            && self.worlds.is_some()
            && self.favorite_worlds.is_some()
            && self.avatars.is_some()
    }
}

fn counts_from_results(
    mutual_friends: Option<Result<usize>>,
    groups: Result<usize>,
    worlds: Result<usize>,
    favorite_worlds: Result<usize>,
    avatars: Result<usize>,
) -> UserDialogTabCountsOutput {
    UserDialogTabCountsOutput {
        mutual_friends: mutual_friends.and_then(|result| resolved_count("mutual friends", result)),
        groups: resolved_count("groups", groups),
        worlds: resolved_count("worlds", worlds),
        favorite_worlds: resolved_count("favorite worlds", favorite_worlds),
        avatars: resolved_count("avatars", avatars),
    }
}

fn resolved_count(source: &str, result: Result<usize>) -> Option<u32> {
    match result {
        Ok(count) => Some(crate::wire_count(count)),
        Err(error) => {
            tracing::debug!(%error, source, "user dialog tab count source failed");
            None
        }
    }
}

#[derive(Deserialize)]
struct FavoriteGroupCountRow {
    #[serde(default)]
    name: Value,
    #[serde(default, rename = "type")]
    kind: Value,
}

fn world_favorite_group_names(payload: &str) -> Result<Vec<String>> {
    let rows = serde_json::from_str::<Vec<FavoriteGroupCountRow>>(payload)?;
    Ok(rows
        .into_iter()
        .filter(|row| row.kind.as_str() == Some("world"))
        .filter_map(|row| {
            row.name
                .as_str()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string)
        })
        .collect())
}

#[derive(Deserialize)]
struct AvatarSearchCountRow {
    #[serde(
        default,
        alias = "Id",
        alias = "_id",
        alias = "avatarId",
        alias = "AvatarId"
    )]
    id: Value,
    #[serde(default, rename = "authorId", alias = "AuthorId", alias = "author_id")]
    author_id: Value,
}

fn count_target_avatars(payload: &str, user_id: &str) -> Result<usize> {
    struct TargetAvatarCountVisitor<'a> {
        user_id: &'a str,
    }

    impl<'de> Visitor<'de> for TargetAvatarCountVisitor<'_> {
        type Value = usize;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a JSON avatar array")
        }

        fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut avatar_ids = HashSet::new();
            while let Some(row) = sequence.next_element::<AvatarSearchCountRow>()? {
                let author_id = row.author_id.as_str().map(str::trim).unwrap_or_default();
                if author_id != self.user_id {
                    continue;
                }
                let avatar_id = row.id.as_str().map(str::trim).unwrap_or_default();
                if !avatar_id.is_empty() {
                    avatar_ids.insert(avatar_id.to_string());
                }
            }
            Ok(avatar_ids.len())
        }
    }

    let mut deserializer = serde_json::Deserializer::from_str(payload);
    Ok(deserializer.deserialize_seq(TargetAvatarCountVisitor { user_id })?)
}

#[derive(Deserialize)]
struct MyAvatarCountRow {
    #[serde(default, rename = "releaseStatus")]
    release_status: Value,
}

fn count_my_avatars(payload: &str, release_status: AvatarReleaseStatus) -> Result<usize> {
    let rows = serde_json::from_str::<Vec<MyAvatarCountRow>>(payload)?;
    if release_status == AvatarReleaseStatus::All {
        return Ok(rows.len());
    }
    Ok(rows
        .iter()
        .filter(|row| row.release_status.as_str() == Some(release_status.as_str()))
        .count())
}

async fn count_payload_pages_bounded<F, Fut, C>(
    page_size: usize,
    max_offset: i32,
    fetch_page: F,
    count_page: C,
) -> Result<usize>
where
    F: FnMut(i32) -> Fut,
    Fut: Future<Output = Result<String>>,
    C: Fn(&str) -> Result<(usize, usize)>,
{
    let mut fetch_page = fetch_page;
    let mut count = 0;
    let mut offset = 0;
    loop {
        let payload = fetch_page(offset).await?;
        let (page_len, selected_count) = count_page(&payload)?;
        count += selected_count;
        if page_len < page_size || offset >= max_offset {
            return Ok(count);
        }
        offset += page_size as i32;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn keeps_successful_counts_when_one_source_fails() {
        let counts = counts_from_results(
            Some(Ok(7)),
            Ok(12),
            Err(Error::Custom("worlds failed".into())),
            Ok(34),
            Ok(56),
        );

        assert_eq!(
            counts,
            UserDialogTabCountsOutput {
                mutual_friends: Some(7),
                groups: Some(12),
                worlds: None,
                favorite_worlds: Some(34),
                avatars: Some(56),
            }
        );
    }

    #[test]
    fn favorite_group_parser_only_keeps_named_world_groups() {
        let payload = serde_json::json!([
            { "name": "worlds1", "type": "world" },
            { "name": "avatars1", "type": "avatar" },
            { "name": "  worlds2  ", "type": "world" },
            { "name": "", "type": "world" }
        ])
        .to_string();

        assert_eq!(
            world_favorite_group_names(&payload).unwrap(),
            vec!["worlds1".to_string(), "worlds2".to_string()]
        );
    }

    #[test]
    fn avatar_count_deduplicates_ids_and_keeps_only_the_target_author() {
        let payload = serde_json::json!([
            { "id": "avtr_1", "authorId": "usr_target" },
            { "Id": "avtr_1", "AuthorId": "usr_target" },
            { "avatarId": "avtr_2", "author_id": "usr_target" },
            { "id": "avtr_3", "authorId": "usr_other" }
        ])
        .to_string();

        assert_eq!(count_target_avatars(&payload, "usr_target").unwrap(), 2);
    }

    #[test]
    fn my_avatar_count_matches_the_selected_release_status() {
        let payload = serde_json::json!([
            { "id": "avtr_public", "releaseStatus": "public" },
            { "id": "avtr_private", "releaseStatus": "private" },
            { "id": "avtr_public_2", "releaseStatus": "public" }
        ])
        .to_string();

        assert_eq!(
            count_my_avatars(&payload, AvatarReleaseStatus::All).unwrap(),
            3
        );
        assert_eq!(
            count_my_avatars(&payload, AvatarReleaseStatus::Public).unwrap(),
            2
        );
        assert_eq!(
            count_my_avatars(&payload, AvatarReleaseStatus::Private).unwrap(),
            1
        );
    }

    #[test]
    fn tab_counts_input_uses_the_release_status_enum() {
        let default_input = serde_json::from_value::<UserDialogTabCountsInput>(
            serde_json::json!({ "userId": "usr_target" }),
        )
        .unwrap();
        assert_eq!(
            default_input.avatar_release_status,
            AvatarReleaseStatus::All
        );

        let public_input = serde_json::from_value::<UserDialogTabCountsInput>(serde_json::json!({
            "userId": "usr_target",
            "avatarReleaseStatus": "public"
        }))
        .unwrap();
        assert_eq!(
            public_input.avatar_release_status,
            AvatarReleaseStatus::Public
        );

        assert!(
            serde_json::from_value::<UserDialogTabCountsInput>(serde_json::json!({
                "userId": "usr_target",
                "avatarReleaseStatus": "invalid"
            }))
            .is_err()
        );
    }

    #[tokio::test]
    async fn page_counter_stops_on_a_short_page_and_uses_monotonic_offsets() {
        let payloads = Arc::new(Mutex::new(VecDeque::from([
            Ok(serde_json::json!([{ "id": 1 }, { "id": 2 }]).to_string()),
            Ok(serde_json::json!([{ "id": 3 }]).to_string()),
        ])));
        let offsets = Arc::new(Mutex::new(Vec::new()));

        let count = count_payload_pages_bounded(
            2,
            100,
            {
                let payloads = Arc::clone(&payloads);
                let offsets = Arc::clone(&offsets);
                move |offset| {
                    offsets.lock().unwrap().push(offset);
                    let payload = payloads.lock().unwrap().pop_front().unwrap();
                    std::future::ready(payload)
                }
            },
            count_all_rows,
        )
        .await
        .unwrap();

        assert_eq!(count, 3);
        assert_eq!(*offsets.lock().unwrap(), vec![0, 2]);
    }

    #[tokio::test]
    async fn bounded_page_counter_includes_the_page_at_the_maximum_offset() {
        let offsets = Arc::new(Mutex::new(Vec::new()));

        let count = count_payload_pages_bounded(
            2,
            4,
            {
                let offsets = Arc::clone(&offsets);
                move |offset| {
                    offsets.lock().unwrap().push(offset);
                    std::future::ready(Ok(
                        serde_json::json!([{ "id": offset }, { "id": offset + 1 }]).to_string(),
                    ))
                }
            },
            count_all_rows,
        )
        .await
        .unwrap();

        assert_eq!(count, 6);
        assert_eq!(*offsets.lock().unwrap(), vec![0, 2, 4]);
    }

    #[tokio::test]
    async fn runtime_reuses_complete_counts_until_force_refresh() {
        let runtime = UserDialogTabCountsRuntime::new();
        let key = UserDialogTabCountsCacheKey {
            auth_generation: 7,
            current_user_id: "usr_self".into(),
            endpoint: "https://api.example.test".into(),
            target_user_id: "usr_target".into(),
            avatar_release_status: "public".into(),
            avatar_provider: "provider-a".into(),
            include_mutual_friends: true,
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let expected = UserDialogTabCountsOutput {
            mutual_friends: Some(5),
            groups: Some(1),
            worlds: Some(2),
            favorite_worlds: Some(3),
            avatars: Some(4),
        };

        for force in [false, false, true] {
            let calls = Arc::clone(&calls);
            let loaded_counts = expected.clone();
            assert_eq!(
                runtime
                    .resolve(key.clone(), force, true, move || async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(loaded_counts)
                    })
                    .await
                    .unwrap(),
                expected
            );
        }

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
