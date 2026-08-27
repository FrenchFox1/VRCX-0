use std::sync::Arc;

use vrcx_0_application::auth::{
    probe_current_user_from_cookie, probe_saved_current_user_from_cookie, record_login_success,
    record_logout, saved_credential_login_start, saved_credential_session_data, saved_snapshot,
    AuthCredentialStore, AuthRemoteRequests, LoginSuccessRecordInput, LogoutRecordInput,
    NonInteractiveAuthActions, NonInteractiveAuthProbeFuture, NonInteractiveAuthResponseFuture,
    SavedAuthSnapshot, SavedCredentialLoginStartInput, SavedCredentialSessionData,
};
use vrcx_0_application_core::{Result, WebClient};

pub struct LocalNonInteractiveAuthActions {
    web: Arc<WebClient>,
    requests: Arc<dyn AuthRemoteRequests>,
    credentials: Arc<dyn AuthCredentialStore>,
}

impl LocalNonInteractiveAuthActions {
    pub fn new(
        web: Arc<WebClient>,
        requests: Arc<dyn AuthRemoteRequests>,
        credentials: Arc<dyn AuthCredentialStore>,
    ) -> Self {
        Self {
            web,
            requests,
            credentials,
        }
    }
}

impl NonInteractiveAuthActions for LocalNonInteractiveAuthActions {
    fn clear_vrchat_config_snapshot(&self) {
        self.web.clear_vrchat_config_snapshot();
    }

    fn saved_snapshot(&self) -> Result<SavedAuthSnapshot> {
        saved_snapshot(self.credentials.as_ref())
    }

    fn saved_session_data(&self, user_id: &str) -> Result<Option<SavedCredentialSessionData>> {
        saved_credential_session_data(self.credentials.as_ref(), user_id)
    }

    fn probe_current_user<'a>(
        &'a self,
        user_id: String,
        endpoint: String,
        websocket: String,
    ) -> NonInteractiveAuthProbeFuture<'a> {
        let web = Arc::clone(&self.web);
        let requests = Arc::clone(&self.requests);
        Box::pin(async move {
            probe_current_user_from_cookie(web, requests, user_id, endpoint, websocket).await
        })
    }

    fn restore_cookies(&self, cookies: &str) -> Result<()> {
        self.web.set_cookies(cookies)
    }

    fn probe_saved_current_user<'a>(
        &'a self,
        user_id: String,
        endpoint: String,
        websocket: String,
    ) -> NonInteractiveAuthProbeFuture<'a> {
        let web = Arc::clone(&self.web);
        let requests = Arc::clone(&self.requests);
        Box::pin(async move {
            probe_saved_current_user_from_cookie(web, requests, user_id, endpoint, websocket).await
        })
    }

    fn start_saved_credential_login<'a>(
        &'a self,
        input: SavedCredentialLoginStartInput,
    ) -> NonInteractiveAuthResponseFuture<'a> {
        let web = Arc::clone(&self.web);
        let requests = Arc::clone(&self.requests);
        Box::pin(async move {
            saved_credential_login_start(self.credentials.as_ref(), web, requests, input).await
        })
    }

    fn record_login_success(&self, input: LoginSuccessRecordInput) -> Result<()> {
        record_login_success(self.credentials.as_ref(), self.web.as_ref(), input).map(|_| ())
    }

    fn clear_browser_session(&self) {
        self.web.clear_cookies();
        self.web.save_cookies();
    }

    fn record_logout(&self, input: LogoutRecordInput) -> Result<()> {
        record_logout(self.credentials.as_ref(), self.web.as_ref(), input).map(|_| ())
    }
}
