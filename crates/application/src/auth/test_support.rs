use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::json;
use vrcx_0_application_core::{vrchat_api::VrchatApiRequest, Error, Result};

use super::{AuthCredentialStore, AuthRemoteRequests, SealedAuthSecret};

#[derive(Clone, Default)]
pub(super) struct MemoryAuthCredentialStore {
    values: Arc<Mutex<HashMap<String, String>>>,
}

impl AuthCredentialStore for MemoryAuthCredentialStore {
    fn get_raw(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .values
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(key)
            .cloned())
    }

    fn get_string(&self, key: &str, default_value: &str) -> Result<String> {
        Ok(self
            .get_raw(key)?
            .unwrap_or_else(|| default_value.to_string()))
    }

    fn get_bool(&self, key: &str, default_value: bool) -> Result<bool> {
        Ok(self
            .get_raw(key)?
            .and_then(|value| value.parse().ok())
            .unwrap_or(default_value))
    }

    fn set_string(&self, key: &str, value: &str) -> Result<()> {
        self.values
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<()> {
        self.values
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(key);
        Ok(())
    }

    fn is_encrypting_writes(&self) -> bool {
        false
    }
    fn is_secret_store_initialized(&self) -> bool {
        false
    }
    fn open_secret(&self, stored: &str) -> Option<String> {
        Some(stored.to_string())
    }
    fn seal_secret(&self, plaintext: &str) -> SealedAuthSecret {
        SealedAuthSecret {
            stored: plaintext.to_string(),
            encrypted: false,
        }
    }
    fn is_sealed_secret(&self, _value: &str) -> bool {
        false
    }
}

pub(super) struct TestAuthRemoteRequests;

fn request(
    endpoint: String,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> VrchatApiRequest {
    VrchatApiRequest {
        endpoint: Some(endpoint),
        method: Some(method.to_string()),
        path: Some(path.to_string()),
        body: body.map_or_default(vrcx_0_contracts::VrchatRequestBody::Json),
        ..Default::default()
    }
}

impl AuthRemoteRequests for TestAuthRemoteRequests {
    fn config(&self, endpoint: String) -> VrchatApiRequest {
        request(endpoint, "GET", "config", None)
    }

    fn current_user(&self, endpoint: String) -> VrchatApiRequest {
        request(endpoint, "GET", "auth/user", None)
    }

    fn basic_login(
        &self,
        endpoint: String,
        username: String,
        password: String,
        username_required: &'static str,
        password_required: &'static str,
    ) -> Result<VrchatApiRequest> {
        if username.trim().is_empty() {
            return Err(Error::Custom(username_required.into()));
        }
        if password.is_empty() {
            return Err(Error::Custom(password_required.into()));
        }
        Ok(request(
            endpoint,
            "GET",
            "auth/user",
            Some(json!({ "username": username, "password": password })),
        ))
    }

    fn verify_totp(&self, endpoint: String, code: String) -> VrchatApiRequest {
        request(
            endpoint,
            "POST",
            "auth/twofactorauth/totp/verify",
            Some(json!({ "code": code })),
        )
    }

    fn verify_email_otp(&self, endpoint: String, code: String) -> VrchatApiRequest {
        request(
            endpoint,
            "POST",
            "auth/twofactorauth/emailotp/verify",
            Some(json!({ "code": code })),
        )
    }

    fn verify_otp(&self, endpoint: String, code: String) -> VrchatApiRequest {
        let normalized_code = code.trim().replace(char::is_whitespace, "");
        let formatted_code = if normalized_code.contains('-') {
            normalized_code
        } else {
            let mut chars = normalized_code.chars();
            let prefix = chars.by_ref().take(4).collect::<String>();
            let suffix = chars.collect::<String>();
            if suffix.is_empty() {
                prefix
            } else {
                format!("{prefix}-{suffix}")
            }
        };
        request(
            endpoint,
            "POST",
            "auth/twofactorauth/otp/verify",
            Some(json!({ "code": formatted_code })),
        )
    }
}
