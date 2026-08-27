use std::path::Path;
use std::sync::Arc;

use vrcx_0_application_core::{InstanceRosterObserver, InstanceRosterSnapshot};
use vrcx_0_application_game::LogLocationSnapshotScanner;
pub use vrcx_0_application_game::{
    GameLogEvent, GameLogEventOrigin, GameLogEventSink, LogLocationSnapshot, LogWatcher,
};

#[derive(Default)]
pub struct HostLogLocationSnapshotScanner;

impl LogLocationSnapshotScanner for HostLogLocationSnapshotScanner {
    fn scan_current_location_snapshot(&self, log_dir: &Path) -> Option<LogLocationSnapshot> {
        vrcx_0_host_desktop::log_scanner::scan_current_location_snapshot(log_dir)
    }
}

pub struct HostInstanceRosterFanout {
    observers: Vec<Arc<dyn InstanceRosterObserver>>,
}

impl HostInstanceRosterFanout {
    pub fn new(observers: Vec<Arc<dyn InstanceRosterObserver>>) -> Self {
        Self { observers }
    }
}

impl InstanceRosterObserver for HostInstanceRosterFanout {
    fn on_instance_roster(&self, snapshot: InstanceRosterSnapshot) {
        for observer in &self.observers {
            observer.on_instance_roster(snapshot.clone());
        }
    }

    fn on_game_running(&self, running: bool) {
        for observer in &self.observers {
            observer.on_game_running(running);
        }
    }
}
