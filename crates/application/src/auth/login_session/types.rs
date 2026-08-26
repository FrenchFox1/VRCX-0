use futures_util::future::BoxFuture;

use std::sync::Arc;

use serde::Serialize;
use vrcx_0_application_core::vrchat_api::{
    VrchatApiRequest as HttpApiRequestInput, VrchatApiResponse as HttpApiExecuteResponse,
    VrchatScope as ApiScope,
};
pub use vrcx_0_core::TwoFactorMethod;

use crate::auth::{AuthenticatedRuntimeSession, SavedAuthSnapshot};
use vrcx_0_application_core::{Result, WebClient};

pub(crate) type LoginApiFuture<'a> = BoxFuture<'a, Result<HttpApiExecuteResponse>>;

pub(crate) trait LoginApi: Send + Sync {
    fn execute<'a>(&'a self, input: HttpApiRequestInput, scope: ApiScope) -> LoginApiFuture<'a>;
    fn config(&self, endpoint: String) -> HttpApiRequestInput;
    fn current_user(&self, endpoint: String) -> HttpApiRequestInput;
    fn basic_login(
        &self,
        endpoint: String,
        username: String,
        password: String,
        username_required: &'static str,
        password_required: &'static str,
    ) -> Result<HttpApiRequestInput>;
    fn verify_totp(&self, endpoint: String, code: String) -> HttpApiRequestInput;
    fn verify_email_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput;
    fn verify_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput;
}

pub trait AuthRemoteRequests: Send + Sync {
    fn config(&self, endpoint: String) -> HttpApiRequestInput;
    fn current_user(&self, endpoint: String) -> HttpApiRequestInput;
    fn basic_login(
        &self,
        endpoint: String,
        username: String,
        password: String,
        username_required: &'static str,
        password_required: &'static str,
    ) -> Result<HttpApiRequestInput>;
    fn verify_totp(&self, endpoint: String, code: String) -> HttpApiRequestInput;
    fn verify_email_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput;
    fn verify_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput;
}

pub(crate) struct WebClientLoginApi {
    web: Arc<WebClient>,
    requests: Arc<dyn AuthRemoteRequests>,
}

impl WebClientLoginApi {
    pub(crate) fn new(web: Arc<WebClient>, requests: Arc<dyn AuthRemoteRequests>) -> Self {
        Self { web, requests }
    }
}

impl LoginApi for WebClientLoginApi {
    fn execute<'a>(&'a self, input: HttpApiRequestInput, scope: ApiScope) -> LoginApiFuture<'a> {
        Box::pin(async move { self.web.execute_api(input, scope).await })
    }

    fn config(&self, endpoint: String) -> HttpApiRequestInput {
        self.requests.config(endpoint)
    }

    fn current_user(&self, endpoint: String) -> HttpApiRequestInput {
        self.requests.current_user(endpoint)
    }

    fn basic_login(
        &self,
        endpoint: String,
        username: String,
        password: String,
        username_required: &'static str,
        password_required: &'static str,
    ) -> Result<HttpApiRequestInput> {
        self.requests.basic_login(
            endpoint,
            username,
            password,
            username_required,
            password_required,
        )
    }

    fn verify_totp(&self, endpoint: String, code: String) -> HttpApiRequestInput {
        self.requests.verify_totp(endpoint, code)
    }

    fn verify_email_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput {
        self.requests.verify_email_otp(endpoint, code)
    }

    fn verify_otp(&self, endpoint: String, code: String) -> HttpApiRequestInput {
        self.requests.verify_otp(endpoint, code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum LoginFailureKind {
    InvalidCredentials,
    MissingCredentials,
    SessionInvalidated,
    TwoFactorUnavailable,
    Network,
    Other,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum LoginSessionState {
    Authenticated {
        session: AuthenticatedRuntimeSession,
        #[serde(skip_serializing_if = "Option::is_none")]
        snapshot: Option<Box<SavedAuthSnapshot>>,
    },
    Challenge {
        #[serde(rename = "attemptId")]
        attempt_id: String,
        #[specta(type = Vec<String>)]
        methods: Vec<TwoFactorMethod>,
        #[specta(type = String)]
        mode: TwoFactorMethod,
        error: Option<String>,
    },
    Failed {
        reason: String,
        kind: LoginFailureKind,
        #[serde(skip_serializing_if = "Option::is_none")]
        snapshot: Option<Box<SavedAuthSnapshot>>,
    },
    Cancelled,
}

impl LoginSessionState {
    pub(super) fn failed(reason: impl Into<String>, kind: LoginFailureKind) -> Self {
        Self::Failed {
            reason: reason.into(),
            kind,
            snapshot: None,
        }
    }
}
