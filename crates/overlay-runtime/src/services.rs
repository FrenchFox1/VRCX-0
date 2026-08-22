use std::sync::Arc;

use vrcx_0_application_activity::notification::{
    load_overlay_activity_filters, NotificationConfig, NotificationRemote,
};
use vrcx_0_application_activity::{
    OverlayActivityRuntime, OverlayActivitySink, OverlayActivitySinkRegistry,
};
use vrcx_0_application_core::{
    HostSessionRuntime, ImageCache, RuntimeAuthScope, TaskSupervisor, WebClient, WorldCache,
};
use vrcx_0_application_game::RuntimeSnapshot;
use vrcx_0_persistence::config::ConfigRepository;
use vrcx_0_persistence::DatabaseService;

pub struct VrOverlayRuntimeDataDeps {
    pub db: Arc<DatabaseService>,
    pub web: Arc<WebClient>,
    pub image_cache: Arc<ImageCache>,
    pub config: ConfigRepository,
    pub notification_config: Arc<dyn NotificationConfig>,
    pub notification_remote: Arc<dyn NotificationRemote>,
    pub auth_scope: RuntimeAuthScope,
    pub session: HostSessionRuntime,
    pub world_cache: Arc<WorldCache>,
    pub tasks: TaskSupervisor,
    pub overlay_activity: OverlayActivityRuntime,
    pub overlay_activity_sinks: OverlayActivitySinkRegistry,
}

pub struct VrOverlayRuntimeData {
    pub(crate) db: Arc<DatabaseService>,
    pub(crate) web: Arc<WebClient>,
    pub(crate) image_cache: Arc<ImageCache>,
    pub(crate) config: ConfigRepository,
    notification_config: Arc<dyn NotificationConfig>,
    #[allow(dead_code)]
    pub(crate) notification_remote: Arc<dyn NotificationRemote>,
    pub(crate) auth_scope: RuntimeAuthScope,
    pub(crate) session: HostSessionRuntime,
    pub(crate) world_cache: Arc<WorldCache>,
    pub(crate) tasks: TaskSupervisor,
    #[allow(dead_code)]
    pub(crate) overlay_activity: OverlayActivityRuntime,
    overlay_activity_sinks: OverlayActivitySinkRegistry,
}

impl VrOverlayRuntimeData {
    pub fn new(deps: VrOverlayRuntimeDataDeps) -> Self {
        Self {
            db: deps.db,
            web: deps.web,
            image_cache: deps.image_cache,
            config: deps.config,
            notification_config: deps.notification_config,
            notification_remote: deps.notification_remote,
            auth_scope: deps.auth_scope,
            session: deps.session,
            world_cache: deps.world_cache,
            tasks: deps.tasks,
            overlay_activity: deps.overlay_activity,
            overlay_activity_sinks: deps.overlay_activity_sinks,
        }
    }

    pub fn config(&self) -> &ConfigRepository {
        &self.config
    }

    pub fn database(&self) -> &Arc<DatabaseService> {
        &self.db
    }

    pub fn web_client(&self) -> &Arc<WebClient> {
        &self.web
    }

    pub fn image_cache(&self) -> &Arc<ImageCache> {
        &self.image_cache
    }

    pub fn auth_scope(&self) -> &RuntimeAuthScope {
        &self.auth_scope
    }

    pub fn session(&self) -> &HostSessionRuntime {
        &self.session
    }

    pub fn world_cache(&self) -> &Arc<WorldCache> {
        &self.world_cache
    }

    pub fn tasks(&self) -> &TaskSupervisor {
        &self.tasks
    }

    pub fn overlay_activity(&self) -> OverlayActivityRuntime {
        self.overlay_activity.clone()
    }

    pub fn add_overlay_activity_sink(&self, sink: Arc<dyn OverlayActivitySink>) {
        self.overlay_activity_sinks.add(sink);
    }

    pub fn reload_overlay_activity_filters(&self) {
        self.overlay_activity
            .set_filters(load_overlay_activity_filters(
                self.notification_config.as_ref(),
            ));
    }
}

pub trait VrOverlayRuntimeServices: Send + Sync {
    fn data(&self) -> &VrOverlayRuntimeData;

    fn game_log_snapshot(&self) -> RuntimeSnapshot;
}
