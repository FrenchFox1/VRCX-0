use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use vrcx_0_contracts::llm::ToolDefinition;
use vrcx_0_persistence::{assistant, config::ConfigRepository, DatabaseService};

use crate::ports::{
    AssistantConfig, AssistantConfigPort, AssistantLlmClientFactory, AssistantLlmClientFactoryPort,
    AssistantLlmClientInput, AssistantLlmClientPort, AssistantLlmError, AssistantMessageInsert,
    AssistantPortError, AssistantPortResult, AssistantSessionPersistence,
    AssistantSessionPersistencePort, AssistantSessionRuntimeUpdate, AssistantSessionUpsert,
    AssistantSqliteErrorCategory, PersistedAssistantMessage, PersistedAssistantSession,
};
use vrcx_0_core::OwnerId;

static TEST_DATABASE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn tool_def(name: &str, parameters: serde_json::Value) -> ToolDefinition {
    ToolDefinition {
        name: name.into(),
        description: String::new(),
        parameters,
    }
}

pub(crate) fn unique_test_database_path(prefix: &str) -> PathBuf {
    let sequence = TEST_DATABASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{}-{nonce}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("VRCX-0.sqlite3")
}

pub(crate) fn test_config_port(prefix: &str) -> AssistantConfig {
    let db = Arc::new(DatabaseService::new(&unique_test_database_path(prefix)).unwrap());
    Arc::new(TestAssistantConfigAdapter {
        config: ConfigRepository::new(db),
    })
}

pub(crate) fn test_session_persistence(db: Arc<DatabaseService>) -> AssistantSessionPersistence {
    Arc::new(TestAssistantSessionPersistenceAdapter { db })
}

pub(crate) fn test_llm_factory() -> AssistantLlmClientFactory {
    Arc::new(TestAssistantLlmClientFactory)
}

struct TestAssistantLlmClientFactory;

impl AssistantLlmClientFactoryPort for TestAssistantLlmClientFactory {
    fn create(
        &self,
        _input: AssistantLlmClientInput,
    ) -> Result<Arc<dyn AssistantLlmClientPort>, AssistantLlmError> {
        Err(AssistantLlmError::NotConfigured)
    }
}

struct TestAssistantConfigAdapter {
    config: ConfigRepository,
}

impl AssistantConfigPort for TestAssistantConfigAdapter {
    fn get_bool(&self, key: &str, default: bool) -> AssistantPortResult<bool> {
        self.config.get_bool(key, default).map_err(port_error)
    }

    fn set_bool(&self, key: &str, value: bool) -> AssistantPortResult<()> {
        self.config.set_bool(key, value).map_err(port_error)
    }

    fn get_string(&self, key: &str, default: &str) -> AssistantPortResult<String> {
        self.config.get_string(key, default).map_err(port_error)
    }

    fn set_string(&self, key: &str, value: &str) -> AssistantPortResult<()> {
        self.config.set_string(key, value).map_err(port_error)
    }

    fn get_json(
        &self,
        key: &str,
        default: serde_json::Value,
    ) -> AssistantPortResult<serde_json::Value> {
        self.config.get_json(key, default).map_err(port_error)
    }

    fn set_json(&self, key: &str, value: &serde_json::Value) -> AssistantPortResult<()> {
        self.config.set_json(key, value).map_err(port_error)
    }
}

struct TestAssistantSessionPersistenceAdapter {
    db: Arc<DatabaseService>,
}

impl AssistantSessionPersistencePort for TestAssistantSessionPersistenceAdapter {
    fn load_sessions(
        &self,
        owner_user_id: &OwnerId,
    ) -> AssistantPortResult<Vec<PersistedAssistantSession>> {
        assistant::assistant_sessions_load(self.db.as_ref(), owner_user_id)
            .map(|sessions| {
                sessions
                    .into_iter()
                    .map(|session| PersistedAssistantSession {
                        owner_user_id: session.owner_user_id,
                        id: session.id,
                        title: session.title,
                        created_at: session.created_at,
                        updated_at: session.updated_at,
                        entity_panel_open: session.entity_panel_open,
                        surfaced_entities: session.surfaced_entities,
                        endpoint_id: session.endpoint_id,
                        model: session.model,
                        allow_writes: session.allow_writes,
                        playbook_mode: session.playbook_mode,
                    })
                    .collect()
            })
            .map_err(port_error)
    }

    fn load_messages(
        &self,
        owner_user_id: &OwnerId,
        session_id: &str,
    ) -> AssistantPortResult<Vec<PersistedAssistantMessage>> {
        assistant::assistant_session_messages_load(self.db.as_ref(), owner_user_id, session_id)
            .map(|messages| {
                messages
                    .into_iter()
                    .map(|message| PersistedAssistantMessage {
                        id: message.id,
                        seq: message.seq,
                        role: message.role,
                        content: message.content,
                        created_at: message.created_at,
                    })
                    .collect()
            })
            .map_err(port_error)
    }

    fn upsert_session(&self, input: AssistantSessionUpsert<'_>) -> AssistantPortResult<()> {
        assistant::assistant_session_upsert(
            self.db.as_ref(),
            input.owner_user_id,
            input.id,
            input.title,
            input.created_at,
            input.updated_at,
        )
        .map_err(port_error)
    }

    fn set_ui_state(
        &self,
        session_id: &str,
        entity_panel_open: bool,
        surfaced_entities: &str,
    ) -> AssistantPortResult<()> {
        assistant::assistant_session_set_ui_state(
            self.db.as_ref(),
            session_id,
            entity_panel_open,
            surfaced_entities,
        )
        .map_err(port_error)
    }

    fn set_runtime(&self, input: AssistantSessionRuntimeUpdate<'_>) -> AssistantPortResult<()> {
        assistant::assistant_session_set_runtime(
            self.db.as_ref(),
            input.id,
            input.endpoint_id,
            input.model,
            input.allow_writes,
            input.playbook_mode,
        )
        .map_err(port_error)
    }

    fn delete_session(&self, owner_user_id: &OwnerId, session_id: &str) -> AssistantPortResult<()> {
        assistant::assistant_session_delete(self.db.as_ref(), owner_user_id, session_id)
            .map_err(port_error)
    }

    fn insert_message(&self, input: AssistantMessageInsert<'_>) -> AssistantPortResult<()> {
        assistant::assistant_message_insert(
            self.db.as_ref(),
            input.id,
            input.session_id,
            input.seq,
            input.role,
            input.content,
            input.created_at,
        )
        .map_err(port_error)
    }
}

fn port_error(error: vrcx_0_persistence::Error) -> AssistantPortError {
    match error {
        vrcx_0_persistence::Error::Database(message) => AssistantPortError::Database {
            message,
            sqlite_category: None,
        },
        vrcx_0_persistence::Error::Sqlite { message, category } => AssistantPortError::Database {
            message,
            sqlite_category: category.map(|category| match category {
                vrcx_0_persistence::SqliteErrorCategory::Malformed => {
                    AssistantSqliteErrorCategory::Malformed
                }
                vrcx_0_persistence::SqliteErrorCategory::DiskFull => {
                    AssistantSqliteErrorCategory::DiskFull
                }
                vrcx_0_persistence::SqliteErrorCategory::Locked => {
                    AssistantSqliteErrorCategory::Locked
                }
                vrcx_0_persistence::SqliteErrorCategory::IoError => {
                    AssistantSqliteErrorCategory::IoError
                }
            }),
        },
        vrcx_0_persistence::Error::Io(error) => AssistantPortError::Io(error.to_string()),
        vrcx_0_persistence::Error::Json(error) => AssistantPortError::Json(error.to_string()),
        vrcx_0_persistence::Error::InvalidData(message) => AssistantPortError::InvalidData(message),
        vrcx_0_persistence::Error::Custom(message) => AssistantPortError::Custom(message),
    }
}
