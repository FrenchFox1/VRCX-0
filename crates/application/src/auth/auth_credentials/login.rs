use std::sync::Arc;

use super::storage::AuthCredentialStore;
use vrcx_0_application_core::vrchat_api::{
    VrchatApiResponse as HttpApiExecuteResponse, VrchatScope as ApiScope,
};
use vrcx_0_application_core::WebClient;

use super::compat::normalize_text;
use super::service::sync_saved_credential_cookies;
use super::storage::read_saved_credentials;
use super::types::SavedCredentialLoginStartInput;
use crate::auth::cookie_session::{probe_cookie_session, CookieProbeResult};
use crate::auth::{LoginApi, WebClientLoginApi};
use vrcx_0_application_core::{Error, Result};

pub async fn saved_credential_login_start(
    config: &dyn AuthCredentialStore,
    web: Arc<WebClient>,
    requests: Arc<dyn crate::auth::AuthRemoteRequests>,
    input: SavedCredentialLoginStartInput,
) -> Result<HttpApiExecuteResponse> {
    let api = WebClientLoginApi::new(Arc::clone(&web), requests);
    saved_credential_login_start_with_api(config, web.as_ref(), &api, input).await
}

pub(crate) async fn saved_credential_login_start_with_api(
    config: &dyn AuthCredentialStore,
    web: &WebClient,
    api: &dyn LoginApi,
    input: SavedCredentialLoginStartInput,
) -> Result<HttpApiExecuteResponse> {
    let user_id = normalize_text(input.user_id);
    if user_id.is_empty() {
        return Err(Error::Custom(
            "VrchatAuthSavedCredentialLoginStart requires a user id.".into(),
        ));
    }

    let saved_credentials = read_saved_credentials(config)?;
    let Some(saved_credential) = saved_credentials.get(&user_id) else {
        return Err(Error::Custom(
            "Saved credentials were not found for the requested account.".into(),
        ));
    };

    let username = saved_credential.login_params.username.clone();
    let password = saved_credential
        .login_params
        .password
        .clone()
        .unwrap_or_default();
    if username.trim().is_empty() || password.is_empty() {
        return Err(Error::Custom(
            "The saved account is missing username or password data.".into(),
        ));
    }

    let endpoint = normalize_text(input.endpoint);
    match probe_cookie_session(api, &endpoint, &user_id).await? {
        CookieProbeResult::Authenticated { response, .. }
        | CookieProbeResult::Rejected { response, .. } => return Ok(response),
        CookieProbeResult::UserMismatch { actual_user_id } => {
            sync_saved_credential_cookies(config, web, &actual_user_id)?;
        }
        CookieProbeResult::MissingCredentials(_) | CookieProbeResult::RequiresTwoFactor(_) => {}
    }

    web.clear_cookies();
    if let Some(cookie) = saved_credential.cookies.as_deref() {
        if let Err(error) = web.set_cookies(cookie) {
            tracing::warn!(
                error = %error,
                user_id = %user_id,
                "failed to restore saved cookies before saved credential login; continuing with password login"
            );
        }
    }

    match probe_cookie_session(api, &endpoint, &user_id).await? {
        CookieProbeResult::Authenticated { response, .. }
        | CookieProbeResult::RequiresTwoFactor(response)
        | CookieProbeResult::Rejected { response, .. } => return Ok(response),
        CookieProbeResult::MissingCredentials(_) | CookieProbeResult::UserMismatch { .. } => {}
    }

    let config_response = api
        .execute(api.config(endpoint.clone()), ApiScope::Vrchat)
        .await?;
    if config_response.status != 200 {
        return Ok(config_response);
    }
    let request = api.basic_login(
        endpoint,
        username,
        password,
        "Saved credential login requires username.",
        "Saved credential login requires password.",
    )?;
    api.execute(request, ApiScope::Vrchat).await
}
