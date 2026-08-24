use std::sync::Arc;

use vrcx_0_application::avatars::{
    AvatarCacheStore, AvatarFeedCleanupStore, AvatarRemoteRequests, MyAvatarsStore,
};
use vrcx_0_application_core::vrchat_api::VrchatApiRequest;
use vrcx_0_contracts::{AvatarTagOutput, AvatarTimeSpentOutput};
use vrcx_0_persistence::DatabaseService;

#[derive(Clone)]
pub struct LocalAvatarApplicationAdapter {
    db: Arc<DatabaseService>,
}

impl LocalAvatarApplicationAdapter {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}

impl AvatarFeedCleanupStore for LocalAvatarApplicationAdapter {
    fn purge_avatar_feed(
        &self,
        user_id: String,
        cutoff_date: Option<String>,
    ) -> crate::Result<i64> {
        Ok(vrcx_0_persistence::feed::feed_avatar_purge(
            &self.db,
            user_id,
            cutoff_date,
        )?)
    }

    fn vacuum_if_fragmented(&self) -> crate::Result<bool> {
        Ok(vrcx_0_persistence::maintenance::database_vacuum_if_fragmented(&self.db)?)
    }
}

impl MyAvatarsStore for LocalAvatarApplicationAdapter {
    fn avatar_tags(&self) -> crate::Result<Vec<AvatarTagOutput>> {
        Ok(vrcx_0_persistence::avatars::avatar_tags_list(&self.db)?)
    }

    fn avatar_time_spent(
        &self,
        owner_user_id: String,
    ) -> crate::Result<Vec<AvatarTimeSpentOutput>> {
        Ok(vrcx_0_persistence::avatars::avatar_time_spent_list(
            &self.db,
            owner_user_id,
        )?)
    }
}

impl AvatarCacheStore for LocalAvatarApplicationAdapter {
    fn remove_cached_avatar(&self, avatar_id: String) -> crate::Result<()> {
        Ok(vrcx_0_persistence::avatars::avatar_cache_remove(
            &self.db, avatar_id,
        )?)
    }
}

impl AvatarRemoteRequests for LocalAvatarApplicationAdapter {
    fn avatar_moderations(&self, endpoint: String) -> crate::Result<VrchatApiRequest> {
        Ok(vrcx_0_vrchat_client::avatars::avatar_moderations_get_input(
            endpoint,
        ))
    }

    fn my_avatar_page(
        &self,
        endpoint: String,
        page_size: i32,
        offset: i32,
    ) -> crate::Result<VrchatApiRequest> {
        Ok(
            vrcx_0_vrchat_client::avatars::avatar_list_by_user_get_input(
                vrcx_0_vrchat_client::avatars::AvatarListByUserGetInput {
                    endpoint,
                    user_id: String::new(),
                    user: "me".into(),
                    n: page_size,
                    offset,
                    sort: vrcx_0_vrchat_client::query::AvatarListSort::Updated,
                    order: vrcx_0_vrchat_client::query::QueryOrder::Descending,
                    release_status: vrcx_0_vrchat_client::query::ReleaseStatusFilter::All,
                },
            )?
            .1,
        )
    }
}
