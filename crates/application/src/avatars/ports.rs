use vrcx_0_application_core::vrchat_api::VrchatApiRequest;
use vrcx_0_application_core::Result;
use vrcx_0_contracts::{AvatarTagOutput, AvatarTimeSpentOutput};

pub trait AvatarFeedCleanupStore: Send + Sync {
    fn purge_avatar_feed(&self, user_id: String, cutoff_date: Option<String>) -> Result<i64>;
    fn vacuum_if_fragmented(&self) -> Result<bool>;
}

pub trait MyAvatarsStore: Send + Sync {
    fn avatar_tags(&self) -> Result<Vec<AvatarTagOutput>>;
    fn avatar_time_spent(&self, owner_user_id: String) -> Result<Vec<AvatarTimeSpentOutput>>;
}

pub trait AvatarCacheStore: Send + Sync {
    fn remove_cached_avatar(&self, avatar_id: String) -> Result<()>;
}

pub trait AvatarRemoteRequests: Send + Sync {
    fn avatar_moderations(&self, endpoint: String) -> Result<VrchatApiRequest>;
    fn my_avatar_page(
        &self,
        endpoint: String,
        page_size: i32,
        offset: i32,
    ) -> Result<VrchatApiRequest>;
}
