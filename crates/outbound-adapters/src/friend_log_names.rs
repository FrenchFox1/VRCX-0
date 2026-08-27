use std::{collections::HashMap, sync::Arc};

use serde_json::{json, Value};
use vrcx_0_application::social::FriendLogNameStore;
use vrcx_0_core::json::RawJson;
use vrcx_0_core::OwnerId;
use vrcx_0_persistence::DatabaseService;

#[derive(Clone)]
pub struct LocalFriendLogNameStore {
    db: Arc<DatabaseService>,
}

impl LocalFriendLogNameStore {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}

impl FriendLogNameStore for LocalFriendLogNameStore {
    fn friend_display_names(
        &self,
        owner_user_id: &OwnerId,
        user_ids: &[String],
    ) -> crate::Result<HashMap<String, String>> {
        vrcx_0_persistence::friends::friend_display_names(&self.db, owner_user_id.clone(), user_ids)
            .map_err(crate::map_persistence_error)
    }

    fn game_log_user_stats(
        &self,
        owner_user_id: &OwnerId,
        user_ids: &[String],
    ) -> crate::Result<Value> {
        vrcx_0_persistence::game_log::game_log_query(
            &self.db,
            owner_user_id,
            vrcx_0_persistence::game_log::GameLogQueryInput {
                kind: "allUserStats".into(),
                params: RawJson::from(json!({ "userIds": user_ids })),
            },
        )
        .map_err(crate::map_persistence_error)
    }
}
