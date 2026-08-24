use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use vrcx_0_application_core::{vrchat_api::VrchatApiRequest, Error, Result};
use vrcx_0_contracts::{
    social_aggregates::{FavoriteAction, FavoriteLocalInput, FavoriteOutput},
    CacheEntityInput, FavoriteRow,
};
use vrcx_0_core::{FavoriteEntityKind, OwnerId};

use super::{
    FavoriteCacheKind, FavoriteMoveResult, FavoriteRemoteAddInput, FavoriteRemoteGroupClearInput,
    FavoriteRemoteGroupSaveInput, FavoriteRemoteRequests, FavoriteStore,
};

pub(super) struct TestFavoriteRemoteRequests;

impl FavoriteRemoteRequests for TestFavoriteRemoteRequests {
    fn list(&self, _endpoint: String, _n: i32, _offset: i32) -> VrchatApiRequest {
        VrchatApiRequest::default()
    }

    fn limits(&self, _endpoint: String) -> VrchatApiRequest {
        VrchatApiRequest::default()
    }

    fn favorite_worlds(
        &self,
        _endpoint: String,
        _n: i32,
        _offset: i32,
        _owner_id: String,
        _user_id: String,
        _tag: String,
    ) -> VrchatApiRequest {
        VrchatApiRequest::default()
    }

    fn favorite_avatars(
        &self,
        _endpoint: String,
        _n: i32,
        _offset: i32,
        _tag: String,
    ) -> VrchatApiRequest {
        VrchatApiRequest::default()
    }

    fn world(&self, _endpoint: String, world_id: String) -> Result<(String, VrchatApiRequest)> {
        Ok((world_id, VrchatApiRequest::default()))
    }

    fn avatar(&self, _endpoint: String, avatar_id: String) -> Result<(String, VrchatApiRequest)> {
        Ok((avatar_id, VrchatApiRequest::default()))
    }

    fn user(&self, _endpoint: String, user_id: String) -> Result<(String, VrchatApiRequest)> {
        Ok((user_id, VrchatApiRequest::default()))
    }

    fn add(
        &self,
        _endpoint: String,
        input: FavoriteRemoteAddInput,
    ) -> Result<(String, String, VrchatApiRequest)> {
        Ok((
            input.kind.as_str().to_string(),
            input.entity_id,
            VrchatApiRequest::default(),
        ))
    }

    fn delete(&self, _endpoint: String, object_id: String) -> Result<(String, VrchatApiRequest)> {
        Ok((object_id, VrchatApiRequest::default()))
    }

    fn save_group(
        &self,
        _endpoint: String,
        _current_user_id: String,
        input: FavoriteRemoteGroupSaveInput,
    ) -> Result<(String, VrchatApiRequest)> {
        Ok((input.group, VrchatApiRequest::default()))
    }

    fn clear_group(
        &self,
        _endpoint: String,
        _current_user_id: String,
        input: FavoriteRemoteGroupClearInput,
    ) -> Result<(String, VrchatApiRequest)> {
        Ok((input.group, VrchatApiRequest::default()))
    }
}

#[derive(Clone)]
struct StoredFavorite {
    owner_user_id: Option<String>,
    kind: FavoriteEntityKind,
    entity_id: String,
    group_name: String,
}

#[derive(Default)]
struct TestFavoriteStoreState {
    configs: HashMap<String, serde_json::Value>,
    favorites: Vec<StoredFavorite>,
    avatar_cache_ids: HashSet<String>,
    world_cache_ids: HashSet<String>,
}

#[derive(Default)]
pub(super) struct TestFavoriteStore {
    state: Mutex<TestFavoriteStoreState>,
}

impl TestFavoriteStore {
    fn owner(owner_user_id: Option<&OwnerId>) -> Option<String> {
        owner_user_id.map(|owner| owner.as_str().to_string())
    }

    fn cache_id(entry: &CacheEntityInput) -> String {
        entry.id.as_str().unwrap_or_default().trim().to_string()
    }
}

impl FavoriteStore for TestFavoriteStore {
    fn config_json(&self, key: &str, fallback: serde_json::Value) -> Result<serde_json::Value> {
        Ok(self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .configs
            .get(key)
            .cloned()
            .unwrap_or(fallback))
    }

    fn set_config_json(&self, key: &str, value: serde_json::Value) -> Result<()> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .configs
            .insert(key.to_string(), value);
        Ok(())
    }

    fn resolve_config_key(&self, key: &str) -> String {
        key.to_string()
    }

    fn list(
        &self,
        owner_user_id: Option<&OwnerId>,
        kind: FavoriteEntityKind,
    ) -> Result<Vec<FavoriteRow>> {
        let owner = Self::owner(owner_user_id);
        Ok(self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .favorites
            .iter()
            .filter(|row| {
                row.kind == kind && (row.owner_user_id == owner || row.owner_user_id.is_none())
            })
            .map(|row| {
                FavoriteRow::new(
                    row.kind,
                    String::new(),
                    row.entity_id.clone(),
                    row.group_name.clone(),
                )
            })
            .collect())
    }

    fn add(
        &self,
        owner_user_id: Option<&OwnerId>,
        kind: FavoriteEntityKind,
        entity_id: String,
        group_name: String,
    ) -> Result<i64> {
        let owner_user_id = Self::owner(owner_user_id);
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.favorites.iter().any(|row| {
            row.owner_user_id == owner_user_id
                && row.kind == kind
                && row.entity_id == entity_id
                && row.group_name == group_name
        }) {
            return Ok(0);
        }
        state.favorites.push(StoredFavorite {
            owner_user_id,
            kind,
            entity_id,
            group_name,
        });
        Ok(1)
    }

    fn remove(
        &self,
        owner_user_id: Option<&OwnerId>,
        kind: FavoriteEntityKind,
        entity_id: String,
        group_name: String,
    ) -> Result<i64> {
        let owner_user_id = Self::owner(owner_user_id);
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let before = state.favorites.len();
        state.favorites.retain(|row| {
            !(row.owner_user_id == owner_user_id
                && row.kind == kind
                && row.entity_id == entity_id
                && row.group_name == group_name)
        });
        Ok((before - state.favorites.len()) as i64)
    }

    fn move_between_groups(
        &self,
        owner_user_id: Option<&OwnerId>,
        kind: FavoriteEntityKind,
        entity_id: String,
        source_group_name: String,
        target_group_name: String,
    ) -> Result<FavoriteMoveResult> {
        let removed = self.remove(owner_user_id, kind, entity_id.clone(), source_group_name)?;
        let added = self.add(owner_user_id, kind, entity_id, target_group_name)?;
        Ok(FavoriteMoveResult { removed, added })
    }

    fn rename_group_with_config(
        &self,
        owner_user_id: Option<&OwnerId>,
        kind: FavoriteEntityKind,
        config_key: &str,
        group_name: &str,
        new_group_name: &str,
        groups: &[String],
    ) -> Result<i64> {
        self.set_config_json(config_key, serde_json::json!(groups))?;
        let owner_user_id = if kind == FavoriteEntityKind::Friend && !config_key.contains(':') {
            None
        } else {
            Self::owner(owner_user_id)
        };
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let mut affected = 0;
        for row in &mut state.favorites {
            if row.owner_user_id == owner_user_id
                && row.kind == kind
                && row.group_name == group_name
            {
                row.group_name = new_group_name.to_string();
                affected += 1;
            }
        }
        Ok(affected)
    }

    fn delete_group_with_config(
        &self,
        owner_user_id: Option<&OwnerId>,
        kind: FavoriteEntityKind,
        config_key: &str,
        group_name: &str,
        groups: &[String],
    ) -> Result<i64> {
        self.set_config_json(config_key, serde_json::json!(groups))?;
        let owner_user_id = if kind == FavoriteEntityKind::Friend && !config_key.contains(':') {
            None
        } else {
            Self::owner(owner_user_id)
        };
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let before = state.favorites.len();
        state.favorites.retain(|row| {
            !(row.owner_user_id == owner_user_id
                && row.kind == kind
                && row.group_name == group_name)
        });
        Ok((before - state.favorites.len()) as i64)
    }

    fn cache_exists(&self, kind: FavoriteCacheKind, id: String) -> Result<bool> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        Ok(match kind {
            FavoriteCacheKind::Avatar => state.avatar_cache_ids.contains(&id),
            FavoriteCacheKind::World => state.world_cache_ids.contains(&id),
        })
    }

    fn cache_upsert(&self, kind: FavoriteCacheKind, entry: CacheEntityInput) -> Result<i64> {
        let id = Self::cache_id(&entry);
        if id.is_empty() {
            return Err(Error::Custom("Favorite cache entry requires id.".into()));
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match kind {
            FavoriteCacheKind::Avatar => state.avatar_cache_ids.insert(id),
            FavoriteCacheKind::World => state.world_cache_ids.insert(id),
        };
        Ok(1)
    }

    fn avatar_cache_existing_ids(&self, avatar_ids: &[String]) -> Result<Vec<String>> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        Ok(avatar_ids
            .iter()
            .filter(|id| state.avatar_cache_ids.contains(*id))
            .cloned()
            .collect())
    }

    fn avatar_cache_upsert_many(&self, entries: Vec<CacheEntityInput>) -> Result<u32> {
        let count = entries.len() as u32;
        for entry in entries {
            self.cache_upsert(FavoriteCacheKind::Avatar, entry)?;
        }
        Ok(count)
    }

    fn mutate_local(
        &self,
        owner_user_id: &OwnerId,
        input: FavoriteLocalInput,
    ) -> Result<FavoriteOutput> {
        let entity_id = input.entity_id.trim().to_string();
        let group = input.group.trim().to_string();
        if entity_id.is_empty() || !entity_id.starts_with(input.kind.entity_id_prefix()) {
            return Err(Error::Custom("favorite requires a valid entity id".into()));
        }
        if group.is_empty() {
            return Err(Error::Custom("favorite requires group".into()));
        }
        let affected_rows = if input.dry_run {
            0
        } else {
            match input.action {
                FavoriteAction::Add => self.add(
                    Some(owner_user_id),
                    input.kind,
                    entity_id.clone(),
                    group.clone(),
                )?,
                FavoriteAction::Remove => self.remove(
                    Some(owner_user_id),
                    input.kind,
                    entity_id.clone(),
                    group.clone(),
                )?,
            }
        };
        Ok(FavoriteOutput {
            kind: input.kind,
            entity_id,
            group,
            action: input.action,
            dry_run: input.dry_run,
            affected_rows,
            caveats: Vec::new(),
        })
    }
}
