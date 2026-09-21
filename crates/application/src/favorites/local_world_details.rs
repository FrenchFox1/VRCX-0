use std::collections::HashSet;

use futures_util::{stream, StreamExt};
use serde::Serialize;
use vrcx_0_application_core::{FavoriteEntityKind, Result, WebClient, WorldCache};
use vrcx_0_core::OwnerId;

use super::local_favorites::{list_local_favorites, FavoriteStore};

const LOCAL_WORLD_DETAILS_REFRESH_CONCURRENCY: usize = 3;

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct LocalWorldDetailsRefreshOutput {
    pub requested: u32,
    pub refreshed: u32,
}

pub async fn refresh_local_world_details(
    store: &dyn FavoriteStore,
    owner_user_id: &OwnerId,
    world_cache: &WorldCache,
    web: &WebClient,
    endpoint: &str,
) -> Result<LocalWorldDetailsRefreshOutput> {
    let mut seen = HashSet::new();
    let world_ids = list_local_favorites(store, owner_user_id, FavoriteEntityKind::World)?
        .into_iter()
        .filter_map(|row| row.world_id)
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty() && seen.insert(id.clone()))
        .collect::<Vec<_>>();
    let requested = world_ids.len() as u32;
    let refreshed = stream::iter(world_ids)
        .map(|world_id| async move {
            match world_cache.get(web, endpoint, &world_id, true, false).await {
                Ok(response) => (200..=299).contains(&response.status),
                Err(error) => {
                    tracing::warn!(world_id, "local favorite world refresh failed: {error}");
                    false
                }
            }
        })
        .buffer_unordered(LOCAL_WORLD_DETAILS_REFRESH_CONCURRENCY)
        .filter(|ok| std::future::ready(*ok))
        .count()
        .await as u32;
    Ok(LocalWorldDetailsRefreshOutput {
        requested,
        refreshed,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vrcx_0_application_core::{MemoryWorldCachePort, NoopWebClientPort};

    use super::*;
    use crate::favorites::test_support::TestFavoriteStore;

    #[tokio::test]
    async fn refreshes_every_local_world_favorite_and_counts_successful_fetches() {
        let store = TestFavoriteStore::default();
        let owner = OwnerId::new("usr_owner".to_string());
        for (world_id, group) in [
            ("wrld_fresh", "Favorites"),
            ("wrld_fresh", "Other"),
            ("wrld_gone", "Favorites"),
        ] {
            store
                .add(
                    Some(&owner),
                    FavoriteEntityKind::World,
                    world_id.to_string(),
                    group.to_string(),
                )
                .unwrap();
        }
        let port = MemoryWorldCachePort::default();
        port.insert(json!({
            "id": "wrld_fresh",
            "name": "Fresh World",
            "imageUrl": "https://example.test/fresh.png",
            "releaseStatus": "private"
        }));
        let world_cache = WorldCache::new(port);
        let web = WebClient::new(NoopWebClientPort);

        let output = refresh_local_world_details(
            &store,
            &owner,
            &world_cache,
            &web,
            "https://api.vrchat.cloud/api/1",
        )
        .await
        .unwrap();

        assert_eq!(output.requested, 2);
        assert_eq!(output.refreshed, 1);
    }
}
