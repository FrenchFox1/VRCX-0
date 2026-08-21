use super::*;
use serde_json::Value;
use std::{path::PathBuf, sync::Condvar, time::Duration};
use vrcx_0_application_core::RuntimeEventSink;
use vrcx_0_persistence::storage::StorageService;

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vrcx0-shared-import-contract-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn running_inner() -> SharedCollectionImportRuntimeInner {
    SharedCollectionImportRuntimeInner {
        status: SharedCollectionImportStatus {
            run_id: "run-1".into(),
            status: SharedCollectionImportState::Running,
            total: 2,
            ..Default::default()
        },
        cancel: Some(Arc::new(AtomicBool::new(false))),
        auth_generation: 1,
    }
}

#[test]
fn auth_scope_switch_marks_active_run_cancelling() {
    let mut inner = running_inner();
    let scope = RuntimeAuthScopeSnapshot {
        generation: 2,
        active: true,
        ..Default::default()
    };

    assert!(mark_cancelling_if_scope_mismatch(&mut inner, &scope));
    assert_eq!(inner.status.status, SharedCollectionImportState::Cancelling);
    assert!(inner.cancel.as_ref().unwrap().load(Ordering::Acquire));
}

#[test]
fn same_auth_scope_hydration_keeps_active_run_running() {
    let auth_scope = RuntimeAuthScope::new();
    let first = auth_scope.set("usr_self", "https://api.vrchat.cloud/api/1");
    let hydrated = auth_scope.set(" usr_self ", "https://api.vrchat.cloud/api/1/");
    let mut inner = running_inner();
    inner.auth_generation = first.generation;

    assert_eq!(hydrated.generation, first.generation);
    assert!(!mark_cancelling_if_scope_mismatch(&mut inner, &hydrated));
    assert_eq!(inner.status.status, SharedCollectionImportState::Running);
    assert!(!inner.cancel.as_ref().unwrap().load(Ordering::Acquire));
}

#[test]
fn status_snapshot_retains_running_progress_for_hydration() {
    let mut inner = running_inner();
    inner.status.processed = 1;
    inner.status.imported = 1;

    let hydrated = inner.status.clone();

    assert_eq!(hydrated.run_id, "run-1");
    assert_eq!(hydrated.processed, 1);
    assert_eq!(hydrated.imported, 1);
    assert_eq!(hydrated.total, 2);
}

#[test]
fn cancelled_terminal_with_imports_is_prepared_once() {
    let mut inner = running_inner();
    let result = SharedCollectionImportResult {
        total: 2,
        processed: 1,
        imported: 1,
        cancelled: true,
        ..Default::default()
    };

    let terminal = prepare_terminal_result(&inner, "run-1", Ok(result.clone()));
    assert!(commit_terminal_status(
        &mut inner,
        "run-1",
        terminal.as_ref().unwrap().status.clone()
    ));
    let duplicate = prepare_terminal_result(&inner, "run-1", Ok(result));

    assert_eq!(
        terminal.as_ref().unwrap().status.status,
        SharedCollectionImportState::Cancelled
    );
    assert_eq!(terminal.unwrap().status.imported, 1);
    assert!(duplicate.is_none());
}

#[derive(Clone, Default)]
struct BlockingFavoritesSink {
    state: Arc<(Mutex<BlockingFavoritesSinkState>, Condvar)>,
}

#[derive(Default)]
struct BlockingFavoritesSinkState {
    entered: bool,
    released: bool,
}

impl BlockingFavoritesSink {
    fn wait_until_entered(&self) {
        let (state, changed) = self.state.as_ref();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let mut state = state.lock().unwrap();
        while !state.entered {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            assert!(!remaining.is_zero(), "favoritesChanged was not emitted");
            let (next, timeout) = changed.wait_timeout(state, remaining).unwrap();
            state = next;
            assert!(
                !timeout.timed_out() || state.entered,
                "favoritesChanged was not emitted"
            );
        }
    }

    fn release(&self) {
        let (state, changed) = self.state.as_ref();
        let mut state = state.lock().unwrap();
        state.released = true;
        changed.notify_all();
    }
}

impl RuntimeEventSink for BlockingFavoritesSink {
    fn emit(&self, event: &str, _payload: Value) {
        if event != "favoritesChanged" {
            return;
        }
        let (state, changed) = self.state.as_ref();
        let mut state = state.lock().unwrap();
        state.entered = true;
        changed.notify_all();
        while !state.released {
            state = changed.wait(state).unwrap();
        }
    }
}

#[test]
fn terminal_status_is_committed_after_favorites_changed_delivery() {
    let dir = TestDir::new("lifecycle");
    let db = Arc::new(DatabaseService::new(&dir.0.join("VRCX-0.sqlite3")).unwrap());
    let storage = StorageService::new(&dir.0.join("storage.json")).unwrap();
    let web = Arc::new(
        WebClient::new(
            &storage,
            db.as_ref(),
            "wss://pipeline.vrchat.cloud".into(),
            "2.2.0",
        )
        .unwrap(),
    );
    let world_cache = Arc::new(WorldCache::new(Arc::clone(&db), 8, Duration::from_secs(60)));
    let event_bus = RuntimeEventBus::new();
    let blocking_sink = BlockingFavoritesSink::default();
    event_bus.set_sink(blocking_sink.clone());
    let tasks = TaskSupervisor::new();
    let auth_scope = RuntimeAuthScope::new();
    auth_scope.set("usr_current", "https://api.vrchat.cloud/api/1");
    let remote_mutations = Arc::new(vrcx_0_application_core::RemoteMutationGate::default());
    let favorite_mutations = FavoriteMutationCoordinator::new(
        Arc::clone(&db),
        Arc::clone(&web),
        vrcx_0_application_core::RuntimeDiagnostics::new(),
        vrcx_0_application_core::RuntimeSyncEngine::new(),
        event_bus.clone(),
        auth_scope.clone(),
        remote_mutations,
    );
    let runtime = SharedCollectionImportRuntime::new(
        db,
        web,
        world_cache,
        event_bus.clone(),
        tasks.clone(),
        auth_scope,
        favorite_mutations,
    )
    .with_test_runner(Arc::new(|prepared, cancel| {
        Box::pin(async move {
            while !cancel.load(Ordering::Acquire) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok(SharedCollectionImportResult {
                total: prepared.world_ids.len(),
                processed: 1,
                imported: 1,
                cancelled: true,
                ..Default::default()
            })
        })
    }));
    let input = SharedCollectionImportStartInput {
        world_ids: vec!["wrld_11111111-1111-1111-1111-111111111111".into()],
        group_name: "Imported worlds".into(),
    };

    let running = runtime.start(input.clone()).unwrap();
    assert_eq!(running.status, SharedCollectionImportState::Running);
    assert!(runtime
        .start(input)
        .unwrap_err()
        .to_string()
        .contains("already active"));

    let cancelling = runtime.cancel();
    assert_eq!(cancelling.status, SharedCollectionImportState::Cancelling);
    blocking_sink.wait_until_entered();
    assert_eq!(
        runtime.status().status,
        SharedCollectionImportState::Cancelling
    );
    blocking_sink.release();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while is_active_status(runtime.status().status) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }

    let terminal = runtime.status();
    assert_eq!(terminal.status, SharedCollectionImportState::Cancelled);
    assert_eq!(terminal.imported, 1);
    let mut events = Vec::new();
    let event_deadline = std::time::Instant::now() + Duration::from_secs(2);
    while events.len() < 4 && std::time::Instant::now() < event_deadline {
        events.extend(event_bus.take_events_for_test());
        if events.len() < 4 {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    assert_eq!(
        events
            .iter()
            .map(|event| event.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "sharedCollectionImportStatus",
            "sharedCollectionImportStatus",
            "favoritesChanged",
            "sharedCollectionImportStatus"
        ]
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.name == "favoritesChanged")
            .count(),
        1
    );
    tasks.stop_all();
}
