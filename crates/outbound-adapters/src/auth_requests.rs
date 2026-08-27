use vrcx_0_application::auth::AuthRemoteRequests;
use vrcx_0_application_core::{vrchat_api::VrchatApiRequest, Result};
use vrcx_0_vrchat_client::auth::{
    config_get_input, current_user_get_input, email_otp_verify_input, login_basic_input,
    otp_verify_input, totp_verify_input,
};

pub struct VrchatAuthRemoteRequests;

impl AuthRemoteRequests for VrchatAuthRemoteRequests {
    fn config(&self, endpoint: String) -> VrchatApiRequest {
        config_get_input(endpoint)
    }

    fn current_user(&self, endpoint: String) -> VrchatApiRequest {
        current_user_get_input(endpoint)
    }

    fn basic_login(
        &self,
        endpoint: String,
        username: String,
        password: String,
        username_required: &'static str,
        password_required: &'static str,
    ) -> Result<VrchatApiRequest> {
        login_basic_input(
            endpoint,
            username,
            password,
            username_required,
            password_required,
        )
        .map(|(_, request)| request)
        .map_err(Into::into)
    }

    fn verify_totp(&self, endpoint: String, code: String) -> VrchatApiRequest {
        totp_verify_input(endpoint, code)
    }

    fn verify_email_otp(&self, endpoint: String, code: String) -> VrchatApiRequest {
        email_otp_verify_input(endpoint, code)
    }

    fn verify_otp(&self, endpoint: String, code: String) -> VrchatApiRequest {
        otp_verify_input(endpoint, code)
    }
}
