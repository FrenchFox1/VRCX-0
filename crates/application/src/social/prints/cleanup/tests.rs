use super::{
    clamp_print_limit, favorite_limit_for_print_limit, is_print_created_content_refresh,
    print_list_items_from_json, select_prints_to_delete, CleanupWarningKind, PrintCleanupDeps,
    PrintCleanupQueue, PrintCleanupTrigger, PrintListItem, PRINT_CLEANUP_DEBOUNCE,
};
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use vrcx_0_application_core::{
    RemoteMutationGate, RuntimeAuthScope, RuntimeTask, RuntimeTaskExecutor, RuntimeTaskHandle,
    TaskSupervisor,
};
use vrcx_0_core::realtime::RealtimeWsMessagePayload;

fn item(id: &str, created_at: &str) -> PrintListItem {
    PrintListItem {
        id: id.to_string(),
        created_at: created_at.to_string(),
    }
}

fn favorite(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|id| (*id).to_string()).collect()
}

fn payload(json: serde_json::Value) -> RealtimeWsMessagePayload {
    RealtimeWsMessagePayload {
        json,
        raw: String::new(),
        received_at: "2026-06-29T00:00:00Z".to_string(),
    }
}

#[test]
fn deletes_oldest_non_favorite_prints_until_limit() {
    let prints = (0..33)
        .map(|index| {
            item(
                &format!("prnt_{index:02}"),
                &format!("2026-06-29T01:{index:02}:00Z"),
            )
        })
        .collect::<Vec<_>>();

    let selection = select_prints_to_delete(&prints, 30, &HashSet::new());

    assert_eq!(selection.to_delete, vec!["prnt_00", "prnt_01", "prnt_02"]);
    assert_eq!(selection.remaining, 30);
    assert_eq!(selection.warning, None);
}

#[test]
fn skips_favorite_prints_even_when_they_are_oldest() {
    let mut prints = vec![item("prnt_favorite", "2026-06-29T00:00:00Z")];
    prints.extend((0..32).map(|index| {
        item(
            &format!("prnt_deletable_{index:02}"),
            &format!("2026-06-29T00:{index:02}:00Z"),
        )
    }));

    let selection = select_prints_to_delete(&prints, 30, &favorite(&["prnt_favorite"]));

    assert_eq!(
        selection.to_delete,
        vec![
            "prnt_deletable_00",
            "prnt_deletable_01",
            "prnt_deletable_02"
        ]
    );
    assert_eq!(selection.remaining, 30);
    assert_eq!(selection.warning, None);
}

#[test]
fn warns_when_favorite_count_exceeds_the_favorite_limit() {
    let prints = (0..27)
        .map(|index| item(&format!("prnt_{index:02}"), "2026-06-29T00:00:00Z"))
        .collect::<Vec<_>>();
    let favorite_ids = prints
        .iter()
        .map(|print| print.id.as_str())
        .collect::<Vec<_>>();

    let selection = select_prints_to_delete(&prints, 30, &favorite(&favorite_ids));

    assert!(selection.to_delete.is_empty());
    assert_eq!(selection.remaining, 27);
    assert_eq!(
        selection.warning.map(|warning| warning.kind),
        Some(CleanupWarningKind::TooManyFavorites)
    );
}

#[test]
fn clamps_print_limit_to_the_supported_range() {
    assert_eq!(clamp_print_limit(1), 30);
    assert_eq!(clamp_print_limit(45), 45);
    assert_eq!(clamp_print_limit(64), 60);
    assert_eq!(favorite_limit_for_print_limit(60), 55);
}

#[test]
fn parses_print_list_items_from_vrchat_json() {
    let items = print_list_items_from_json(&json!([
        { "id": "prnt_a", "createdAt": "2026-06-29T00:00:00Z" },
        { "id": "prnt_b", "timestamp": "2026-06-29T01:00:00Z" },
        { "id": "", "createdAt": "2026-06-29T02:00:00Z" },
        { "name": "missing id" }
    ]));

    assert_eq!(
        items,
        vec![
            item("prnt_a", "2026-06-29T00:00:00Z"),
            item("prnt_b", "2026-06-29T01:00:00Z")
        ]
    );
}

#[test]
fn detects_print_created_content_refresh_messages() {
    assert!(is_print_created_content_refresh(&payload(json!({
        "type": "content-refresh",
        "content": {
            "contentType": "print",
            "actionType": "created"
        }
    }))));
    assert!(!is_print_created_content_refresh(&payload(json!({
        "type": "content-refresh",
        "content": {
            "contentType": "print",
            "actionType": "deleted"
        }
    }))));
    assert!(!is_print_created_content_refresh(&payload(json!({
        "type": "friend-online",
        "content": {
            "contentType": "print",
            "actionType": "created"
        }
    }))));
}

#[test]
fn cleanup_queue_uses_2500ms_debounce_and_keeps_one_flight_pending() {
    let supervisor = TaskSupervisor::new();
    let executor = CountingTaskExecutor::default();
    let spawned = Arc::clone(&executor.spawned);
    supervisor.set_executor(executor);
    let queue = PrintCleanupQueue::new();
    let _dir = TestDir::new("print-cleanup-queue");
    let deps = test_deps(&_dir.path);
    let trigger = PrintCleanupTrigger {
        user_id: "usr_self".into(),
        endpoint: "https://api.vrchat.cloud/api/1".into(),
        reason: "test".into(),
    };

    assert_eq!(PRINT_CLEANUP_DEBOUNCE, Duration::from_millis(2500));
    queue.schedule(&supervisor, deps.clone(), trigger.clone());
    queue.schedule(&supervisor, deps.clone(), trigger.clone());
    queue.schedule(&supervisor, deps, trigger);

    assert_eq!(spawned.load(Ordering::Acquire), 1);
}

#[derive(Clone, Default)]
struct CountingTaskExecutor {
    spawned: Arc<AtomicUsize>,
}

struct CountingTaskHandle {
    finished: bool,
}

impl RuntimeTaskExecutor for CountingTaskExecutor {
    fn spawn(&self, _task: RuntimeTask) -> Box<dyn RuntimeTaskHandle> {
        self.spawned.fetch_add(1, Ordering::AcqRel);
        Box::new(CountingTaskHandle { finished: false })
    }
}

impl RuntimeTaskHandle for CountingTaskHandle {
    fn abort(&self) {}

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn join_or_abort(&mut self, _timeout: Duration) {
        self.finished = true;
    }
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("vrcx-0-{name}-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn test_deps(path: &std::path::Path) -> PrintCleanupDeps {
    let db =
        Arc::new(vrcx_0_persistence::DatabaseService::new(&path.join("VRCX-0.sqlite3")).unwrap());
    let storage =
        vrcx_0_persistence::storage::StorageService::new(&path.join("storage.json")).unwrap();
    let web = Arc::new(
        crate::WebClient::new(
            &storage,
            db.as_ref(),
            "wss://pipeline.vrchat.cloud".into(),
            env!("CARGO_PKG_VERSION"),
        )
        .unwrap(),
    );
    PrintCleanupDeps {
        db,
        web,
        event_bus: vrcx_0_application_core::RuntimeEventBus::new(),
        auth_scope: RuntimeAuthScope::new(),
        remote_mutations: Arc::new(RemoteMutationGate::default()),
    }
}
