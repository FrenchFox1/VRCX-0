use vrcx_0_application::favorites::{
    FavoriteRemoteAddInput, FavoriteRemoteGroupClearInput, FavoriteRemoteGroupSaveInput,
    FavoriteRemoteRequests,
};
use vrcx_0_application_core::{vrchat_api::VrchatApiRequest, Result};
use vrcx_0_vrchat_client::avatars::avatar_get_input;
use vrcx_0_vrchat_client::favorites::{
    favorite_add_input, favorite_avatars_get_input, favorite_delete_input,
    favorite_group_clear_input, favorite_group_save_input, favorite_limits_get_input,
    favorite_worlds_get_input, favorites_get_input,
};
use vrcx_0_vrchat_client::users::user_get_input;
use vrcx_0_vrchat_client::worlds::world_get_input;

pub struct VrchatFavoriteRemoteRequests;

impl FavoriteRemoteRequests for VrchatFavoriteRemoteRequests {
    fn list(&self, endpoint: String, n: i32, offset: i32) -> VrchatApiRequest {
        favorites_get_input(endpoint, n, offset)
    }

    fn limits(&self, endpoint: String) -> VrchatApiRequest {
        favorite_limits_get_input(endpoint)
    }

    fn favorite_worlds(
        &self,
        endpoint: String,
        n: i32,
        offset: i32,
        owner_id: String,
        user_id: String,
        tag: String,
    ) -> VrchatApiRequest {
        favorite_worlds_get_input(endpoint, n, offset, owner_id, user_id, tag)
    }

    fn favorite_avatars(
        &self,
        endpoint: String,
        n: i32,
        offset: i32,
        tag: String,
    ) -> VrchatApiRequest {
        favorite_avatars_get_input(endpoint, n, offset, tag)
    }

    fn world(&self, endpoint: String, world_id: String) -> Result<(String, VrchatApiRequest)> {
        world_get_input(endpoint, world_id).map_err(Into::into)
    }

    fn avatar(&self, endpoint: String, avatar_id: String) -> Result<(String, VrchatApiRequest)> {
        avatar_get_input(endpoint, avatar_id).map_err(Into::into)
    }

    fn user(&self, endpoint: String, user_id: String) -> Result<(String, VrchatApiRequest)> {
        user_get_input(endpoint, user_id).map_err(Into::into)
    }

    fn add(
        &self,
        endpoint: String,
        input: FavoriteRemoteAddInput,
    ) -> Result<(String, String, VrchatApiRequest)> {
        favorite_add_input(endpoint, input.kind, input.entity_id, input.tags).map_err(Into::into)
    }

    fn delete(&self, endpoint: String, object_id: String) -> Result<(String, VrchatApiRequest)> {
        favorite_delete_input(endpoint, object_id).map_err(Into::into)
    }

    fn save_group(
        &self,
        endpoint: String,
        current_user_id: String,
        input: FavoriteRemoteGroupSaveInput,
    ) -> Result<(String, VrchatApiRequest)> {
        favorite_group_save_input(
            endpoint,
            current_user_id,
            input.kind,
            input.group,
            input.display_name,
            input.visibility,
        )
        .map_err(Into::into)
    }

    fn clear_group(
        &self,
        endpoint: String,
        current_user_id: String,
        input: FavoriteRemoteGroupClearInput,
    ) -> Result<(String, VrchatApiRequest)> {
        favorite_group_clear_input(endpoint, current_user_id, input.kind, input.group)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vrcx_0_application::favorites::{FavoriteRemoteAddInput, FavoriteRemoteRequests};
    use vrcx_0_application_core::VrchatFavoriteType;

    use super::VrchatFavoriteRemoteRequests;

    #[test]
    fn vrc_plus_world_add_preserves_request_shape() {
        let (kind, entity_id, request) = VrchatFavoriteRemoteRequests
            .add(
                "endpoint".into(),
                FavoriteRemoteAddInput {
                    kind: VrchatFavoriteType::VrcPlusWorld,
                    entity_id: "wrld_1".into(),
                    tags: "worlds4".into(),
                },
            )
            .unwrap();

        assert_eq!(kind, "vrcPlusWorld");
        assert_eq!(entity_id, "wrld_1");
        assert_eq!(request.path.as_deref(), Some("favorites"));
        assert_eq!(
            request.body.as_json(),
            Some(&json!({
                "type": "vrcPlusWorld",
                "favoriteId": "wrld_1",
                "tags": "worlds4",
            }))
        );
    }
}
