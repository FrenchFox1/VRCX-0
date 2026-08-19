use serde::Deserialize;
use vrcx_0_application_core::vrchat_api::deserialize_nonnegative_i32;
use vrcx_0_application_core::vrchat_api::groups::{
    GroupMemberPatch, GroupMemberSort, GroupPostMutation,
};
use vrcx_0_core::GroupJoinRequestAction;

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupIdInput {
    #[serde(default)]
    pub(super) group_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupProfileInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default = "default_true")]
    pub(super) include_roles: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupUserGroupsInput {
    #[serde(default)]
    pub(super) user_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupPagedInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) offset: i32,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupMembersInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) offset: i32,
    #[serde(default)]
    pub(super) sort: GroupMemberSort,
    #[serde(default)]
    pub(super) role_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupMembersSearchInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) offset: i32,
    #[serde(default)]
    pub(super) query: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupGalleryInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) gallery_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) offset: i32,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupJoinRequestsInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) offset: i32,
    #[serde(default)]
    pub(super) blocked: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupLogsInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    pub(super) offset: i32,
    #[serde(default)]
    pub(super) event_types: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupPostCreateInput {
    #[serde(default)]
    pub(super) group_id: String,
    pub(super) params: GroupPostMutation,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupPostEditInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) post_id: String,
    pub(super) params: GroupPostMutation,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupPostDeleteInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) post_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupUserInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) user_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupMemberRoleInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) user_id: String,
    #[serde(default)]
    pub(super) role_id: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupJoinRequestRespondInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) user_id: String,
    pub(super) action: GroupJoinRequestAction,
    #[serde(default)]
    pub(super) block: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupRepresentationInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) is_representing: bool,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatGroupMemberPropsInput {
    #[serde(default)]
    pub(super) group_id: String,
    #[serde(default)]
    pub(super) user_id: String,
    pub(super) params: GroupMemberPatch,
}

fn default_true() -> bool {
    true
}
