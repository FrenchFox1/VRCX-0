use std::collections::VecDeque;
use std::sync::Mutex;

use serde_json::{json, Value};
use vrcx_0_application_core::vrchat_api::{
    VrchatApiRequest as HttpApiRequestInput, VrchatApiResponse as HttpApiExecuteResponse,
    VrchatScope as ApiScope,
};
use vrcx_0_contracts::vrchat_api::vrchat_response;

use crate::auth::test_support::{MemoryAuthCredentialStore, TestAuthRemoteRequests};
use crate::auth::{
    record_login_success, AuthCredentialStore, AuthRemoteRequests, LoginSuccessRecordInput,
};
use vrcx_0_application_core::{Error, MemoryCookieWebClientPort, Result, WebClient};

use super::types::{LoginApi, LoginApiFuture};

struct RecordedCall {
    path: String,
    body: Option<Value>,
}

pub(super) struct FakeLoginApi {
    responses: Mutex<VecDeque<std::result::Result<HttpApiExecuteResponse, String>>>,
    calls: Mutex<Vec<RecordedCall>>,
}

impl FakeLoginApi {
    pub(super) fn new(responses: Vec<(i32, Value)>) -> Self {
        Self::new_raw(
            responses
                .into_iter()
                .map(|(status, body)| (status, body.to_string()))
                .collect(),
        )
    }

    pub(super) fn new_raw(responses: Vec<(i32, String)>) -> Self {
        Self {
            responses: Mutex::new(
                responses
                    .into_iter()
                    .map(|(status, body)| Ok(vrchat_response(status, body)))
                    .collect(),
            ),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn with_network_error(mut self, message: &str) -> Self {
        self.responses
            .get_mut()
            .unwrap()
            .push_back(Err(message.to_string()));
        self
    }

    pub(super) fn call_paths(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .map(|call| call.path.clone())
            .collect()
    }

    pub(super) fn call_bodies(&self) -> Vec<Option<Value>> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .map(|call| call.body.clone())
            .collect()
    }
}

impl LoginApi for FakeLoginApi {
    fn execute<'a>(&'a self, input: HttpApiRequestInput, _scope: ApiScope) -> LoginApiFuture<'a> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(RecordedCall {
                path: input.path.clone().unwrap_or_default(),
                body: input.body.as_json().cloned(),
            });
            let next = self
                .responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("test queued too few fake responses");
            next.map_err(Error::Custom)
        })
    }

    fn config(&self, endpoint: String) -> HttpApiRequestInput {
        TestAuthRemoteRequests.config(endpoint)
    }

    fn current_user(&self, endpoint: String) -> HttpApiRequestInput {
        TestAuthRemoteRequests.current_user(endpoint)
    }

    fn basic_login(
        &self,
        endpoint: String,
        username: String,
        password: String,
        username_required: &'static str,
        password_required: &'static str,
    ) -> Result<HttpApiRequestInput> {
        TestAuthRemoteRequests.basic_login(
            endpoint,
            username,
            password,
            username_required,
            password_required,
        )
    }

    fn verify_totp(&self, endpoint: String, code: String) -> HttpApiRequestInput {
        TestAuthRemoteRequests.verify_totp(endpoint, code)
    }

    fn verify_email_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput {
        TestAuthRemoteRequests.verify_email_otp(endpoint, code)
    }

    fn verify_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput {
        TestAuthRemoteRequests.verify_otp(endpoint, code)
    }
}

pub(super) fn user_json() -> Value {
    json!({ "id": "usr_123", "displayName": "Example" })
}

pub(super) struct TestDir {
    path: std::path::PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vrcx-0-login-session-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub(super) fn test_env(name: &str) -> (TestDir, MemoryAuthCredentialStore, WebClient, ()) {
    let dir = TestDir::new(name);
    let config = MemoryAuthCredentialStore::default();
    let web = WebClient::new(MemoryCookieWebClientPort::default());
    (dir, config, web, ())
}

pub(super) fn seed_saved_credential(
    config: &dyn AuthCredentialStore,
    web: &WebClient,
    user_id: &str,
) {
    record_login_success(
        config,
        web,
        LoginSuccessRecordInput {
            user: json!({ "id": user_id, "displayName": "Saved User" }).into(),
            login_params: json!({ "username": "saved@example.test", "password": "secret" }).into(),
            stored_login_params: None,
            save_credentials: true,
        },
    )
    .unwrap();
}
