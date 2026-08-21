use super::*;

use chrono::Weekday;
use std::sync::atomic::AtomicUsize;
use vrcx_0_persistence::DatabaseService;

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vrcx0-telemetry-contract-{name}-{}-{nonce}",
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

fn instant_past_epoch_safe(headroom: Duration) -> Instant {
    Instant::now() + headroom
}

#[test]
fn local_weekday_uses_sunday_zero() {
    assert_eq!(local_weekday_number(Weekday::Sun), 0);
    assert_eq!(local_weekday_number(Weekday::Mon), 1);
    assert_eq!(local_weekday_number(Weekday::Sat), 6);
}

#[test]
fn runtime_mode_maps_all_backend_modes() {
    assert_eq!(
        runtime_mode(BackendRuntimeMode::Foreground),
        TelemetryRuntimeMode::Foreground
    );
    assert_eq!(
        runtime_mode(BackendRuntimeMode::Background),
        TelemetryRuntimeMode::Background
    );
    assert_eq!(
        runtime_mode(BackendRuntimeMode::Headless),
        TelemetryRuntimeMode::Headless
    );
}

#[test]
fn send_attempts_back_off_between_retries() {
    let now = instant_past_epoch_safe(SEND_RETRY_BACKOFF);

    assert!(attempt_due(None, now));
    assert!(!attempt_due(Some(now), now));
    assert!(!attempt_due(
        Some(now - SEND_RETRY_BACKOFF + Duration::from_secs(1)),
        now
    ));
    assert!(attempt_due(Some(now - SEND_RETRY_BACKOFF), now));
}

#[test]
fn heartbeat_waits_for_interval_after_initial_baseline() {
    let now = instant_past_epoch_safe(HEARTBEAT_INTERVAL);

    assert!(!is_heartbeat_due(None, now));
    assert!(!is_heartbeat_due(Some(now), now));
    assert!(!is_heartbeat_due(
        Some(now - HEARTBEAT_INTERVAL + Duration::from_secs(1)),
        now
    ));
    assert!(is_heartbeat_due(Some(now - HEARTBEAT_INTERVAL), now));
}

#[test]
fn theme_mode_category_resolves_system_without_unknown() {
    assert_eq!(theme_mode_category("dark", ""), "dark");
    assert_eq!(theme_mode_category("midnight", ""), "dark");
    assert_eq!(theme_mode_category("light", ""), "light");
    assert_eq!(theme_mode_category("system", "dark"), "dark");
    assert_eq!(theme_mode_category("system", "light"), "light");
    assert_eq!(theme_mode_category("system", ""), "light");
    assert_eq!(theme_mode_category("other", ""), "unknown");
}

#[test]
fn helpers_normalize_config_and_dimension_values() {
    assert_eq!(normalize_enum_value(" On Demand "), "on_demand");
    assert_eq!(normalize_enum_value(""), "unknown");
    assert_eq!(normalize_locale("zh_CN"), "zh-CN");
    assert_eq!(normalize_app_version(""), "unknown");
}

#[test]
fn cursor_acknowledgement_only_clears_the_matching_snapshot() {
    let mut pending = Some("2026-07-13T10:00:00Z".to_string());
    clear_committed_error_cursor(&mut pending, "2026-07-13T09:00:00Z");
    assert_eq!(pending.as_deref(), Some("2026-07-13T10:00:00Z"));

    clear_committed_error_cursor(&mut pending, "2026-07-13T10:00:00Z");
    assert!(pending.is_none());
}

#[tokio::test]
async fn client_error_flush_retries_before_advancing_versioned_log_cursor() {
    let dir = TestDir::new("retry");
    let error_log = (1..=21)
        .map(|day| {
            let version = if day == 1 { "2.0.0" } else { "2.1.0" };
            format!(
                "[2026-07-{day:02} 00:00:00.000 +00:00] [2026-07-{day:02}T00:00:00.000Z] [v{version}] [rust:tracing]\nrelease failure {day}\n\n"
            )
        })
        .collect::<String>();
    std::fs::write(dir.0.join("error-log.txt"), error_log).unwrap();
    let db = Arc::new(DatabaseService::new(&dir.0.join("VRCX-0.sqlite3")).unwrap());
    let config = ConfigRepository::new(db);
    config
        .set_string(
            TELEMETRY_CLIENT_ERROR_CURSOR_CONFIG_KEY,
            "2026-06-30T00:00:00.000Z",
        )
        .unwrap();
    let attempts = Arc::new(AtomicUsize::new(0));
    let payloads = Arc::new(Mutex::new(Vec::new()));
    let runtime = TelemetryRuntime::new_with_test_post(
        TelemetryRuntimeDeps {
            config: config.clone(),
            tasks: TaskSupervisor::new(),
            backend_runtime: BackendRuntime::new(
                vrcx_0_application_core::RuntimeHostProfile::Desktop,
            ),
            app_version: "2.2.0".into(),
            app_data: dir.0.clone(),
            system_theme_category: Arc::new(|| "dark".into()),
        },
        {
            let attempts = attempts.clone();
            let payloads = payloads.clone();
            Arc::new(move |path, payload| {
                let attempts = attempts.clone();
                let payloads = payloads.clone();
                Box::pin(async move {
                    assert_eq!(path, "/api/v1/telemetry/client-error");
                    payloads.lock().unwrap().push(payload);
                    attempts.fetch_add(1, Ordering::SeqCst) != 1
                })
            })
        },
    );
    let session = TelemetrySession {
        install_id: "install".into(),
        session_id: "session".into(),
        is_new_install: false,
    };

    runtime.drain_rust_errors();
    runtime.flush_collectors_locked(&session).await;

    assert_eq!(
        config
            .get_string(TELEMETRY_CLIENT_ERROR_CURSOR_CONFIG_KEY, "")
            .unwrap(),
        "2026-06-30T00:00:00.000Z"
    );
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    let first_attempt_payloads = payloads.lock().unwrap().clone();
    let mut app_versions = first_attempt_payloads
        .iter()
        .flat_map(|payload| payload["errors"].as_array().unwrap())
        .map(|error| error["appVersion"].as_str().unwrap())
        .collect::<Vec<_>>();
    app_versions.sort_unstable();
    assert_eq!(app_versions.len(), 21);
    assert_eq!(app_versions[0], "2.0.0");
    assert!(app_versions[1..].iter().all(|version| *version == "2.1.0"));

    runtime.flush_collectors_locked(&session).await;

    assert_eq!(attempts.load(Ordering::SeqCst), 4);
    assert_eq!(
        config
            .get_string(TELEMETRY_CLIENT_ERROR_CURSOR_CONFIG_KEY, "")
            .unwrap(),
        "2026-07-21T00:00:00.000Z"
    );
    assert!(runtime
        .inner
        .state
        .lock()
        .unwrap()
        .pending_error_cursor
        .is_none());
}

#[tokio::test]
async fn immediate_rust_error_flush_only_sends_sanitized_client_errors() {
    let dir = TestDir::new("immediate-rust-error");
    std::fs::write(
        dir.0.join("error-log.txt"),
        "[2026-07-01 00:00:00.000 +00:00] [2026-07-01T00:00:00.000Z] [v2.2.0] [rust:tracing]\ndatabase upgrade failed: C:\\Users\\alice\\AppData\\secret.sqlite3\n\n",
    )
    .unwrap();
    let db = Arc::new(DatabaseService::new(&dir.0.join("VRCX-0.sqlite3")).unwrap());
    let config = ConfigRepository::new(db);
    let payloads = Arc::new(Mutex::new(Vec::new()));
    let runtime = TelemetryRuntime::new_with_test_post(
        TelemetryRuntimeDeps {
            config: config.clone(),
            tasks: TaskSupervisor::new(),
            backend_runtime: BackendRuntime::new(
                vrcx_0_application_core::RuntimeHostProfile::Desktop,
            ),
            app_version: "2.2.0".into(),
            app_data: dir.0.clone(),
            system_theme_category: Arc::new(|| "dark".into()),
        },
        {
            let payloads = payloads.clone();
            Arc::new(move |path, payload| {
                let payloads = payloads.clone();
                Box::pin(async move {
                    assert_eq!(path, "/api/v1/telemetry/client-error");
                    payloads.lock().unwrap().push(payload);
                    true
                })
            })
        },
    );

    runtime.flush_pending_rust_errors().await;

    let payloads = payloads.lock().unwrap();
    assert_eq!(payloads.len(), 1);
    let encoded = payloads[0].to_string();
    assert!(encoded.contains("database upgrade failed"));
    assert!(!encoded.contains("alice"));
    assert!(!encoded.contains("secret.sqlite3"));
    assert_eq!(
        config
            .get_string(TELEMETRY_CLIENT_ERROR_CURSOR_CONFIG_KEY, "")
            .unwrap(),
        "2026-07-01T00:00:00.000Z"
    );
}

#[tokio::test]
async fn immediate_rust_error_flush_fails_closed_when_consent_is_unavailable() {
    let dir = TestDir::new("unavailable-consent");
    std::fs::write(
        dir.0.join("error-log.txt"),
        "[2026-07-01 00:00:00.000 +00:00] [2026-07-01T00:00:00.000Z] [v2.2.0] [rust:tracing]\ndatabase upgrade failed\n\n",
    )
    .unwrap();
    let db = Arc::new(DatabaseService::new(&dir.0.join("VRCX-0.sqlite3")).unwrap());
    let config = ConfigRepository::new(db.clone());
    let attempts = Arc::new(AtomicUsize::new(0));
    let runtime = TelemetryRuntime::new_with_test_post(
        TelemetryRuntimeDeps {
            config,
            tasks: TaskSupervisor::new(),
            backend_runtime: BackendRuntime::new(
                vrcx_0_application_core::RuntimeHostProfile::Desktop,
            ),
            app_version: "2.2.0".into(),
            app_data: dir.0.clone(),
            system_theme_category: Arc::new(|| "dark".into()),
        },
        {
            let attempts = attempts.clone();
            Arc::new(move |_, _| {
                let attempts = attempts.clone();
                Box::pin(async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    true
                })
            })
        },
    );
    db.freeze_for_migration().unwrap();

    runtime.flush_pending_rust_errors().await;

    assert_eq!(attempts.load(Ordering::SeqCst), 0);
}
