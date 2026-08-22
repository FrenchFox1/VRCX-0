use std::sync::{Arc, Mutex};

use vrcx_0_application_activity::notification::{
    extract_file_id, extract_file_version, fallback_file_version, normalize_avatar_image_url_128,
    CachedNotificationUserImageResolver, RealtimeUserImageResolverSlot,
};
use vrcx_0_application_activity::{OverlayActivityRuntime, OverlayActivitySink};
use vrcx_0_application_core::FriendProjection;
use vrcx_0_application_game::{
    GameLogSideEffectEvent, GameLogSideEffectObserver, NowPlayingSnapshot, RuntimeSnapshot,
    RuntimeSnapshotStore,
};
use vrcx_0_application_realtime::{FriendProjectionObserver, RealtimeHostRuntime};
use vrcx_0_core::friends::StateBucket;
use vrcx_0_host_desktop::tts::{SystemTtsEngine, TtsEngine};
use vrcx_0_overlay_runtime::VrOverlayRuntimeData;
#[cfg(any(windows, target_os = "linux"))]
use vrcx_0_overlay_runtime::VrOverlayRuntimeServices;

use crate::host_actions::RuntimeHost;
use crate::notification::{
    seed_hmd_notifications_default, DesktopNotifier, DesktopNotifierSlot, NotificationDispatcher,
    NotificationDispatcherDeps,
};

const AVATAR_PREFETCH_MAX_PATCHES: usize = 8;

pub struct DesktopRuntimeServices {
    data: Arc<VrOverlayRuntimeData>,
    pub host: RuntimeHost,
    tts: Arc<dyn TtsEngine>,
    notification_desktop_notifier: DesktopNotifierSlot,
    realtime_user_image_resolver: RealtimeUserImageResolverSlot,
    realtime_user_image_resolver_owner: Mutex<Option<Arc<dyn CachedNotificationUserImageResolver>>>,
    game_log_snapshot: RuntimeSnapshotStore,
    now_playing: Arc<Mutex<Arc<NowPlayingSnapshot>>>,
}

impl DesktopRuntimeServices {
    pub fn new(data: Arc<VrOverlayRuntimeData>) -> Self {
        if let Err(error) = seed_hmd_notifications_default(data.config()) {
            tracing::warn!(error = %error, "failed to seed HMD notification preference");
        }
        let tts: Arc<dyn TtsEngine> = Arc::new(SystemTtsEngine::new());
        let notification_desktop_notifier = DesktopNotifierSlot::default();
        let realtime_user_image_resolver = RealtimeUserImageResolverSlot::default();
        let notification_sink: Arc<dyn OverlayActivitySink> =
            Arc::new(NotificationDispatcher::new(NotificationDispatcherDeps {
                session: data.session().clone(),
                auth_scope: data.auth_scope().clone(),
                config: data.config().clone(),
                db: Arc::clone(data.database()),
                image_cache: Arc::clone(data.image_cache()),
                realtime_user_image_resolver: realtime_user_image_resolver.clone(),
                desktop: Arc::new(notification_desktop_notifier.clone()),
                tts: Arc::clone(&tts),
                tasks: data.tasks().clone(),
            }));
        data.add_overlay_activity_sink(notification_sink);
        Self {
            data,
            host: RuntimeHost::new(),
            tts,
            notification_desktop_notifier,
            realtime_user_image_resolver,
            realtime_user_image_resolver_owner: Mutex::new(None),
            game_log_snapshot: RuntimeSnapshotStore::default(),
            now_playing: Arc::new(Mutex::new(Arc::new(NowPlayingSnapshot::default()))),
        }
    }

    pub fn data(&self) -> &VrOverlayRuntimeData {
        self.data.as_ref()
    }

    pub fn reload_overlay_activity_filters(&self) {
        self.data.reload_overlay_activity_filters();
    }

    pub fn set_overlay_activity_extra_sink(&self, extra_sink: Arc<dyn OverlayActivitySink>) {
        self.data.add_overlay_activity_sink(extra_sink);
    }

    pub fn set_notification_desktop_notifier(&self, desktop: Arc<dyn DesktopNotifier>) {
        self.notification_desktop_notifier.set(desktop);
    }

    pub fn set_realtime_user_image_resolver(&self, realtime_runtime: &Arc<RealtimeHostRuntime>) {
        let resolver: Arc<dyn CachedNotificationUserImageResolver> = Arc::new(
            vrcx_0_outbound_adapters::RealtimeNotificationUserImageResolver::new(realtime_runtime),
        );
        self.realtime_user_image_resolver.set(&resolver);
        match self.realtime_user_image_resolver_owner.lock() {
            Ok(mut owner) => *owner = Some(resolver),
            Err(error) => tracing::warn!(
                error = %error,
                "failed to retain realtime notification image resolver"
            ),
        }
    }

    pub fn game_log_snapshot_handle(&self) -> RuntimeSnapshotStore {
        self.game_log_snapshot.clone()
    }

    pub fn game_log_snapshot(&self) -> Arc<RuntimeSnapshot> {
        self.game_log_snapshot.snapshot()
    }

    pub fn now_playing(&self) -> Arc<NowPlayingSnapshot> {
        self.now_playing
            .lock()
            .map(|snapshot| Arc::clone(&snapshot))
            .unwrap_or_else(|_| Arc::new(NowPlayingSnapshot::default()))
    }

    pub fn overlay_activity(&self) -> OverlayActivityRuntime {
        self.data.overlay_activity()
    }

    pub fn tts(&self) -> Arc<dyn TtsEngine> {
        Arc::clone(&self.tts)
    }

    fn observe_game_log_side_effect(&self, event: &GameLogSideEffectEvent) {
        match event {
            GameLogSideEffectEvent::NowPlaying(payload) => match self.now_playing.lock() {
                Ok(mut current) => {
                    Arc::make_mut(&mut current).apply(payload);
                }
                Err(error) => {
                    tracing::warn!("failed to lock now playing snapshot: {error}");
                }
            },
            GameLogSideEffectEvent::NowPlayingReset(_) => match self.now_playing.lock() {
                Ok(mut current) => {
                    *current = Arc::new(NowPlayingSnapshot::default());
                }
                Err(error) => {
                    tracing::warn!("failed to lock now playing snapshot: {error}");
                }
            },
            GameLogSideEffectEvent::ScreenshotProcessed(_)
            | GameLogSideEffectEvent::GameNoVr(_)
            | GameLogSideEffectEvent::Notification(_) => {}
        }
    }

    fn prefetch_online_friend_avatars(&self, projection: &FriendProjection) {
        if projection.patches.len() > AVATAR_PREFETCH_MAX_PATCHES {
            return;
        }
        let Some(endpoint) = self
            .data
            .session()
            .snapshot()
            .realtime_context
            .map(|context| context.endpoint)
            .filter(|endpoint| !endpoint.is_empty())
        else {
            return;
        };
        let allow_user_icon = self
            .data
            .config()
            .get_bool("displayVRCPlusIconsAsAvatar", true)
            .unwrap_or(true);
        for patch in &projection.patches {
            if !StateBucket::Online.matches(&patch.patch.state) {
                continue;
            }
            let user_id = patch.user_id.as_str();
            if !user_id.starts_with("usr_") {
                continue;
            }
            let Some(raw_url) =
                self.realtime_user_image_resolver
                    .cached_url(&endpoint, user_id, allow_user_icon)
            else {
                continue;
            };
            let normalized = normalize_avatar_image_url_128(&raw_url, &endpoint);
            let Some(file_id) = extract_file_id(&normalized) else {
                continue;
            };
            let version = extract_file_version(&normalized, &file_id)
                .unwrap_or_else(|| fallback_file_version(&normalized));
            if version.is_empty() {
                continue;
            }
            let image_cache = Arc::clone(self.data.image_cache());
            self.data.tasks().spawn(async move {
                let _ = image_cache.get_image(&normalized, &file_id, &version).await;
            });
        }
    }
}

impl GameLogSideEffectObserver for DesktopRuntimeServices {
    fn on_game_log_side_effect(&self, event: &GameLogSideEffectEvent) {
        self.observe_game_log_side_effect(event);
    }
}

impl FriendProjectionObserver for DesktopRuntimeServices {
    fn on_friend_projection(&self, projection: &FriendProjection) {
        self.prefetch_online_friend_avatars(projection);
    }
}

#[cfg(any(windows, target_os = "linux"))]
impl VrOverlayRuntimeServices for DesktopRuntimeServices {
    fn data(&self) -> &VrOverlayRuntimeData {
        DesktopRuntimeServices::data(self)
    }

    fn game_log_snapshot(&self) -> RuntimeSnapshot {
        DesktopRuntimeServices::game_log_snapshot(self)
            .as_ref()
            .clone()
    }
}

#[cfg(test)]
mod tests;
