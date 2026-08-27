use vrcx_0_application::game::BackgroundGroupRequests;
use vrcx_0_application_core::vrchat_api::VrchatApiRequest;

#[derive(Clone, Copy, Default)]
pub struct VrchatBackgroundGroupRequests;

impl BackgroundGroupRequests for VrchatBackgroundGroupRequests {
    fn current_user(&self, endpoint: String) -> crate::Result<VrchatApiRequest> {
        Ok(vrcx_0_vrchat_client::auth::current_user_get_input(endpoint))
    }

    fn current_user_group_instances(
        &self,
        endpoint: String,
        current_user_id: String,
    ) -> crate::Result<VrchatApiRequest> {
        Ok(
            vrcx_0_vrchat_client::groups::current_user_group_instances_get_input(
                endpoint,
                current_user_id,
            )?
            .1,
        )
    }

    fn current_user_group_instances_for_group(
        &self,
        endpoint: String,
        current_user_id: String,
        group_id: String,
    ) -> crate::Result<VrchatApiRequest> {
        Ok(
            vrcx_0_vrchat_client::groups::user_group_instances_get_input_for_endpoint(
                endpoint,
                group_id,
                current_user_id,
            )?
            .2,
        )
    }

    fn group_profile(&self, endpoint: String, group_id: String) -> crate::Result<VrchatApiRequest> {
        Ok(vrcx_0_vrchat_client::groups::profile_get_input(endpoint, group_id, false)?.1)
    }
}
