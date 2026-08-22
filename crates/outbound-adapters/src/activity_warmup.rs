use std::sync::Arc;

use vrcx_0_application_activity::{ActivitySessionWarmupOutput, ActivitySessionWarmupStore};
use vrcx_0_application_core::Result;
use vrcx_0_core::OwnerId;
use vrcx_0_persistence::DatabaseService;

use crate::map_persistence_error;

pub struct LocalActivitySessionWarmupStore {
    db: Arc<DatabaseService>,
}

impl LocalActivitySessionWarmupStore {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}

impl ActivitySessionWarmupStore for LocalActivitySessionWarmupStore {
    fn warm_self_sessions(
        &self,
        owner_user_id: OwnerId,
        range_days: i64,
    ) -> Result<ActivitySessionWarmupOutput> {
        let output = vrcx_0_persistence::activity::activity_self_sessions_warmup(
            self.db.as_ref(),
            owner_user_id,
            range_days,
            None,
        )
        .map_err(map_persistence_error)?;
        Ok(ActivitySessionWarmupOutput {
            cached_range_days: output.sync.cached_range_days,
            source_count: output.source_count,
            session_count: output.sessions.len(),
        })
    }
}
