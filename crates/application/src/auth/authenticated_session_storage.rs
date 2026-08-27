use vrcx_0_application_core::Result;

pub trait AuthenticatedSessionStorage: Send + Sync {
    fn ensure_user_scope(&self, user_id: &str) -> Result<()>;
}

pub fn initialize_authenticated_session_storage(
    storage: &dyn AuthenticatedSessionStorage,
    user_id: &str,
) -> Result<()> {
    storage.ensure_user_scope(user_id)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct RecordingStorage {
        user_ids: Mutex<Vec<String>>,
        failure: Option<String>,
    }

    impl AuthenticatedSessionStorage for RecordingStorage {
        fn ensure_user_scope(&self, user_id: &str) -> Result<()> {
            self.user_ids.lock().unwrap().push(user_id.to_string());
            match &self.failure {
                Some(message) => Err(vrcx_0_application_core::Error::Custom(message.clone())),
                None => Ok(()),
            }
        }
    }

    #[test]
    fn initialization_forwards_the_authenticated_user_id_verbatim() {
        let storage = RecordingStorage::default();

        initialize_authenticated_session_storage(&storage, " usr_owner ").unwrap();

        assert_eq!(storage.user_ids.lock().unwrap().as_slice(), [" usr_owner "]);
    }

    #[test]
    fn initialization_preserves_storage_failure() {
        let storage = RecordingStorage {
            failure: Some("schema unavailable".into()),
            ..Default::default()
        };

        let error = initialize_authenticated_session_storage(&storage, "usr_owner").unwrap_err();

        assert_eq!(error.to_string(), "schema unavailable");
    }
}
