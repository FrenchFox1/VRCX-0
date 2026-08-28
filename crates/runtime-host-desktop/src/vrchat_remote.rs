use vrcx_0_application::remote::VrchatApiRuntime;
use vrcx_0_application::remote::{
    AvatarListSort as ApplicationAvatarListSort,
    CalendarListParams as ApplicationCalendarListParams,
    EmojiLoopStyle as ApplicationEmojiLoopStyle, EmojiUploadParams as ApplicationEmojiUploadParams,
    GroupSearchParams as ApplicationGroupSearchParams,
    ImageAnimationStyle as ApplicationImageAnimationStyle, ImageMaskTag as ApplicationImageMaskTag,
    InstanceCreateGroupAccessType as ApplicationInstanceCreateGroupAccessType,
    InstanceCreateMinimumAvatarPerformance as ApplicationInstanceCreateMinimumAvatarPerformance,
    InstanceCreateRegion as ApplicationInstanceCreateRegion,
    InstanceCreateRequest as ApplicationInstanceCreateRequest,
    InstanceCreateType as ApplicationInstanceCreateType,
    InventoryItemUpdateRequest as ApplicationInventoryItemUpdateRequest,
    InventoryListParams as ApplicationInventoryListParams,
    InventoryOrder as ApplicationInventoryOrder, InviteMessageType as ApplicationInviteMessageType,
    MediaAssetUploadRequest as ApplicationMediaAssetUploadRequest,
    MediaFileListParams as ApplicationMediaFileListParams, MediaFileTag as ApplicationMediaFileTag,
    PrintUploadParams as ApplicationPrintUploadParams,
    ProfileDecorationEquipSlot as ApplicationProfileDecorationEquipSlot,
    QueryOrder as ApplicationQueryOrder, ReleaseStatusFilter as ApplicationReleaseStatusFilter,
    RequestInviteRequest as ApplicationRequestInviteRequest,
    UserSearchCustomField as ApplicationUserSearchCustomField,
    UserSearchParams as ApplicationUserSearchParams, UserSearchSort as ApplicationUserSearchSort,
    WorldSearchParams as ApplicationWorldSearchParams,
    WorldSearchSort as ApplicationWorldSearchSort,
};
use vrcx_0_application_core::vrchat_api::{VrchatApiRequest, VrchatApiResponse, VrchatScope};
use vrcx_0_application_core::Result;
use vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT;
use vrcx_0_vrchat_client::auth::{
    current_user_get_input, file_analysis_get_input, visits_get_input,
};
use vrcx_0_vrchat_client::avatars::{
    avatar_file_get_input, avatar_gallery_get_input, avatar_list_by_user_get_input,
    avatar_styles_get_input, AvatarListByUserGetInput,
};
use vrcx_0_vrchat_client::favorites::{favorite_groups_get_input, favorite_worlds_get_input};
use vrcx_0_vrchat_client::friends::friend_status_get_input;
use vrcx_0_vrchat_client::instances::{
    instance_close_input, instance_create_input, instance_get_input, instance_self_invite_input,
    instance_short_name_get_input, InstanceCreateGroupAccessType,
    InstanceCreateMinimumAvatarPerformance, InstanceCreateRegion, InstanceCreateRequest,
    InstanceCreateType,
};
use vrcx_0_vrchat_client::media::{
    asset_upload_input, avatar_gallery_image_upload_input, file_delete_input, files_get_input,
    image_upload_input, inventory_bundle_consume_input, inventory_item_equip_input,
    inventory_item_update_input, inventory_items_get_input, inventory_slot_unequip_input,
    inventory_template_get_input, print_delete_input, print_get_input, print_upload_input,
    prints_get_input, reward_redeem_input, sticker_upload_input, tagged_image_upload_input,
    user_inventory_item_get_input, EmojiLoopStyle, EmojiUploadParams, ImageAnimationStyle,
    ImageMaskTag, InventoryItemUpdateRequest, InventoryListParams, InventoryOrder,
    MediaAssetUploadRequest, MediaFileListParams, MediaFileTag, PrintUploadParams,
    ProfileDecorationEquipSlot,
};
use vrcx_0_vrchat_client::notifications::{
    boop_send_input, request_invite_photo_input, request_invite_send_input, RequestInviteRequest,
};
use vrcx_0_vrchat_client::query::{
    AvatarListSort, QueryOrder, ReleaseStatusFilter, UserSearchCustomField, UserSearchSort,
    WorldSearchSort,
};
use vrcx_0_vrchat_client::search::{
    search_groups_get_input, search_groups_strict_get_input, search_instance_short_name_get_input,
    search_users_get_input, search_worlds_get_input, GroupSearchParams, UserSearchParams,
    WorldSearchParams,
};
use vrcx_0_vrchat_client::tools::{
    following_calendars_get_input, group_calendar_get_input, group_calendar_ics_get_input,
    group_event_follow_input, invite_message_edit_input, invite_messages_get_input,
    user_note_save_input, user_report_input, CalendarListParams, InviteMessageType,
};
use vrcx_0_vrchat_client::users::{profile_get_input, user_represented_group_get_input};

use crate::DesktopMediaRuntime;

#[derive(Clone)]
pub struct DesktopVrchatRemoteFacade {
    api: VrchatApiRuntime,
    media: DesktopMediaRuntime,
}

impl DesktopVrchatRemoteFacade {
    pub(crate) fn new(api: VrchatApiRuntime, media: DesktopMediaRuntime) -> Self {
        Self { api, media }
    }

    pub async fn current_user(&self) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_auth_current_user_get",
            "Getting current VRChat user.",
            current_user_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into()),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn visits(&self) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_auth_visits_get",
            "Getting online visits.",
            visits_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into()),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn file_analysis(
        &self,
        file_id: String,
        version: i64,
        variant: String,
    ) -> Result<VrchatApiResponse> {
        let (file_id, request) = file_analysis_get_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            file_id,
            version,
            variant,
        )?;
        self.execute(
            "app__vrchat_auth_file_analysis_get",
            format!("Getting file analysis for {file_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn user_profile(&self, user_id: String, as_self: bool) -> Result<VrchatApiResponse> {
        let (user_id, request) =
            profile_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), user_id, as_self)?;
        self.execute(
            "app__vrchat_user_profile_get",
            format!("Getting profile for user {user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn user_represented_group(&self, user_id: String) -> Result<VrchatApiResponse> {
        let (user_id, request) =
            user_represented_group_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), user_id)?;
        self.execute(
            "app__vrchat_user_represented_group_get",
            format!("Getting represented group for user {user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn friend_status(&self, user_id: String) -> Result<VrchatApiResponse> {
        let (user_id, request) =
            friend_status_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), user_id)?;
        self.execute(
            "app__vrchat_friend_status_get",
            format!("Getting friend status for {user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn favorite_worlds(
        &self,
        n: i32,
        offset: i32,
        owner_id: String,
        user_id: String,
        tag: String,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_favorite_worlds_get",
            format!("Getting favorite worlds offset {offset}."),
            favorite_worlds_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                n,
                offset,
                owner_id,
                user_id,
                tag,
            ),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn favorite_groups(
        &self,
        n: i32,
        offset: i32,
        owner_id: String,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_favorite_groups_get",
            format!("Getting favorite groups offset {offset}."),
            favorite_groups_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), n, offset, owner_id),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn avatar_gallery(&self, avatar_id: String) -> Result<VrchatApiResponse> {
        let (avatar_id, request) =
            avatar_gallery_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), avatar_id)?;
        self.execute(
            "app__vrchat_avatar_gallery_get",
            format!("Getting avatar gallery for {avatar_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn avatars_by_user(
        &self,
        user_id: String,
        user: String,
        n: i32,
        offset: i32,
        sort: ApplicationAvatarListSort,
        order: ApplicationQueryOrder,
        release_status: ApplicationReleaseStatusFilter,
    ) -> Result<VrchatApiResponse> {
        let (display_user, request) = avatar_list_by_user_get_input(AvatarListByUserGetInput {
            endpoint: VRCHAT_API_DEFAULT_ENDPOINT.into(),
            user_id,
            user,
            n,
            offset,
            sort: avatar_list_sort(sort),
            order: query_order(order),
            release_status: release_status_filter(release_status),
        })?;
        self.execute(
            "app__vrchat_avatar_list_by_user_get",
            format!("Getting avatars for {display_user}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn avatar_styles(&self) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_avatar_styles_get",
            "Getting avatar styles.",
            avatar_styles_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into()),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn avatar_file(&self, file_id: String) -> Result<VrchatApiResponse> {
        let (file_id, request) =
            avatar_file_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), file_id)?;
        self.execute(
            "app__vrchat_avatar_file_get",
            format!("Getting file {file_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn instance_get(
        &self,
        world_id: String,
        instance_id: String,
    ) -> Result<VrchatApiResponse> {
        let (world_id, instance_id, request) =
            instance_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), world_id, instance_id)?;
        self.execute(
            "app__vrchat_instance_get",
            format!("Getting instance {world_id}:{instance_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn instance_short_name(
        &self,
        world_id: String,
        instance_id: String,
        short_name: String,
    ) -> Result<VrchatApiResponse> {
        let (world_id, instance_id, request) = instance_short_name_get_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            world_id,
            instance_id,
            short_name,
        )?;
        self.execute(
            "app__vrchat_instance_short_name_get",
            format!("Getting short name for instance {world_id}:{instance_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn instance_create(
        &self,
        params: ApplicationInstanceCreateRequest,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_instance_create",
            "Creating instance.",
            instance_create_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                instance_create_request(params),
            )?,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn instance_self_invite(
        &self,
        world_id: String,
        instance_id: String,
        short_name: String,
    ) -> Result<VrchatApiResponse> {
        let (world_id, instance_id, request) = instance_self_invite_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            world_id,
            instance_id,
            short_name,
        )?;
        self.execute(
            "app__vrchat_instance_self_invite",
            format!("Sending self invite for {world_id}:{instance_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn instance_close(
        &self,
        location: String,
        hard_close: bool,
    ) -> Result<VrchatApiResponse> {
        let (location, request) =
            instance_close_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), location, hard_close)?;
        self.execute(
            "app__vrchat_instance_close",
            format!("Closing instance {location}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn search_worlds(
        &self,
        params: ApplicationWorldSearchParams,
        option: Option<String>,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_search_worlds_get",
            "Searching worlds.",
            search_worlds_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                world_search_params(params),
                option,
            ),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn search_users(
        &self,
        params: ApplicationUserSearchParams,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_search_users_get",
            "Searching users.",
            search_users_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                user_search_params(params),
            ),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn search_groups(
        &self,
        params: ApplicationGroupSearchParams,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_search_groups_get",
            "Searching groups.",
            search_groups_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                group_search_params(params),
            ),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn search_groups_strict(
        &self,
        params: ApplicationGroupSearchParams,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_search_groups_strict_get",
            "Strict searching groups.",
            search_groups_strict_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                group_search_params(params),
            ),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn search_instance_short_name(
        &self,
        short_name: String,
    ) -> Result<VrchatApiResponse> {
        let (short_name, request) =
            search_instance_short_name_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), short_name)?;
        self.execute(
            "app__vrchat_search_instance_short_name_get",
            format!("Resolving instance short name {short_name}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn group_calendar(&self, group_id: String) -> Result<VrchatApiResponse> {
        let (group_id, request) =
            group_calendar_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), group_id)?;
        self.execute(
            "app__vrchat_tools_group_calendar_get",
            format!("Getting group calendar {group_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn following_calendars(
        &self,
        params: ApplicationCalendarListParams,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_tools_following_calendars_get",
            "Getting followed group calendars.",
            following_calendars_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                calendar_list_params(params),
            ),
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn follow_group_event(
        &self,
        group_id: String,
        event_id: String,
        is_following: bool,
    ) -> Result<VrchatApiResponse> {
        let (event_id, request) = group_event_follow_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            group_id,
            event_id,
            is_following,
        )?;
        self.execute(
            "app__vrchat_tools_group_event_follow",
            format!("Updating follow state for event {event_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn group_calendar_ics(
        &self,
        group_id: String,
        event_id: String,
    ) -> Result<VrchatApiResponse> {
        let (event_id, request) =
            group_calendar_ics_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), group_id, event_id)?;
        self.execute(
            "app__vrchat_tools_group_calendar_ics_get",
            format!("Getting calendar ICS for event {event_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn save_user_note(
        &self,
        target_user_id: String,
        note: String,
    ) -> Result<VrchatApiResponse> {
        let (target_user_id, request) =
            user_note_save_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), target_user_id, note)?;
        self.execute(
            "app__vrchat_tools_user_note_save",
            format!("Saving note for user {target_user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn report_user(&self, user_id: String, reason: String) -> Result<VrchatApiResponse> {
        let (user_id, request) =
            user_report_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), user_id, reason)?;
        self.execute(
            "app__vrchat_tools_user_report",
            format!("Reporting user {user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn invite_messages(
        &self,
        current_user_id: String,
        message_type: ApplicationInviteMessageType,
    ) -> Result<VrchatApiResponse> {
        let (current_user_id, request) = invite_messages_get_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            current_user_id,
            invite_message_type(message_type),
        )?;
        self.execute(
            "app__vrchat_tools_invite_messages_get",
            format!("Getting invite messages for {current_user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn edit_invite_message(
        &self,
        current_user_id: String,
        message_type: ApplicationInviteMessageType,
        slot: i32,
        message: String,
    ) -> Result<VrchatApiResponse> {
        let (slot, request) = invite_message_edit_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            current_user_id,
            invite_message_type(message_type),
            slot,
            message,
        )?;
        self.execute(
            "app__vrchat_tools_invite_message_edit",
            format!("Editing invite message {slot}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn request_invite(
        &self,
        receiver_user_id: String,
        params: ApplicationRequestInviteRequest,
    ) -> Result<VrchatApiResponse> {
        let (receiver_user_id, request) = request_invite_send_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            receiver_user_id,
            request_invite_request(params),
        )?;
        self.execute(
            "app__vrchat_request_invite_send",
            format!("Sending invite request to {receiver_user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn request_invite_photo(
        &self,
        receiver_user_id: String,
        params: ApplicationRequestInviteRequest,
        image_data: String,
    ) -> Result<VrchatApiResponse> {
        let (receiver_user_id, request) = request_invite_photo_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            receiver_user_id,
            request_invite_request(params),
            image_data,
        )?;
        let request = self.media.prepare_media_upload_request(request)?;
        self.execute(
            "app__vrchat_request_invite_photo_send",
            format!("Sending invite request photo to {receiver_user_id}."),
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn boop(&self, user_id: String, emoji_id: String) -> Result<VrchatApiResponse> {
        let (user_id, request) =
            boop_send_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), user_id, emoji_id)?;
        self.execute(
            "app__vrchat_boop_send",
            format!("Sending boop to {user_id}."),
            request,
            VrchatScope::Vrchat,
        )
        .await
    }

    pub async fn media_files(
        &self,
        params: ApplicationMediaFileListParams,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_media_files_get",
            "Getting media files.",
            files_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                media_file_list_params(params),
            ),
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn delete_media_file(&self, file_id: String) -> Result<VrchatApiResponse> {
        let detail = format!("Deleting media file {file_id}.");
        let request = file_delete_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), file_id)?;
        self.execute(
            "app__vrchat_media_file_delete",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn upload_gallery_image(&self, image_data: String) -> Result<VrchatApiResponse> {
        let request = tagged_image_upload_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            image_data,
            "gallery",
            false,
        )?;
        self.execute_media_upload(
            "app__vrchat_media_gallery_image_upload",
            "Uploading gallery image.",
            request,
        )
        .await
    }

    pub async fn upload_avatar_gallery_image(
        &self,
        image_data: String,
        avatar_id: String,
    ) -> Result<VrchatApiResponse> {
        let request = avatar_gallery_image_upload_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            image_data,
            avatar_id,
        )?;
        self.execute_media_upload(
            "app__vrchat_media_avatar_gallery_image_upload",
            "Uploading avatar gallery image.",
            request,
        )
        .await
    }

    pub async fn upload_vrc_plus_icon(&self, image_data: String) -> Result<VrchatApiResponse> {
        let request = tagged_image_upload_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            image_data,
            "icon",
            true,
        )?;
        self.execute_media_upload(
            "app__vrchat_media_vrc_plus_icon_upload",
            "Uploading VRC+ icon.",
            request,
        )
        .await
    }

    pub async fn upload_emoji(
        &self,
        image_data: String,
        params: ApplicationEmojiUploadParams,
    ) -> Result<VrchatApiResponse> {
        let request = image_upload_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            "file/image",
            image_data,
            emoji_upload_params(params),
            true,
        )?;
        self.execute_media_upload(
            "app__vrchat_media_emoji_upload",
            "Uploading emoji.",
            request,
        )
        .await
    }

    pub async fn upload_sticker(&self, image_data: String) -> Result<VrchatApiResponse> {
        let request = sticker_upload_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), image_data)?;
        self.execute_media_upload(
            "app__vrchat_media_sticker_upload",
            "Uploading sticker.",
            request,
        )
        .await
    }

    pub async fn upload_print(
        &self,
        image_data: String,
        crop_white_border: bool,
        params: ApplicationPrintUploadParams,
    ) -> Result<VrchatApiResponse> {
        let request = print_upload_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            image_data,
            crop_white_border,
            print_upload_params(params),
        )?;
        self.execute_media_upload(
            "app__vrchat_media_print_upload",
            "Uploading print.",
            request,
        )
        .await
    }

    pub async fn upload_media_asset(
        &self,
        input: ApplicationMediaAssetUploadRequest,
    ) -> Result<VrchatApiResponse> {
        let (asset_kind, request) = asset_upload_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            media_asset_upload_request(input),
        )?;
        self.execute_media_upload(
            "app__vrchat_media_asset_upload",
            format!("Uploading media asset {asset_kind}."),
            request,
        )
        .await
    }

    pub async fn prints(&self, user_id: String, n: i32) -> Result<VrchatApiResponse> {
        let detail = format!("Getting prints for user {user_id}.");
        let request = prints_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), user_id, n)?;
        self.execute(
            "app__vrchat_media_prints_get",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn print(&self, print_id: String) -> Result<VrchatApiResponse> {
        let detail = format!("Getting print {print_id}.");
        let request = print_get_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), print_id)?;
        self.execute(
            "app__vrchat_media_print_get",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn delete_print(&self, print_id: String) -> Result<VrchatApiResponse> {
        self.media.ensure_print_deletable(&print_id)?;
        let detail = format!("Deleting print {print_id}.");
        let request = print_delete_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), print_id)?;
        self.execute(
            "app__vrchat_media_print_delete",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn inventory_items(
        &self,
        params: ApplicationInventoryListParams,
    ) -> Result<VrchatApiResponse> {
        self.execute(
            "app__vrchat_media_inventory_items_get",
            "Getting inventory items.",
            inventory_items_get_input(
                VRCHAT_API_DEFAULT_ENDPOINT.into(),
                inventory_list_params(params),
            ),
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn inventory_template(
        &self,
        inventory_template_id: String,
    ) -> Result<VrchatApiResponse> {
        let detail = format!("Getting inventory template {inventory_template_id}.");
        let request = inventory_template_get_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            inventory_template_id,
        )?;
        self.execute(
            "app__vrchat_media_inventory_template_get",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn equip_profile_decoration(
        &self,
        inventory_id: String,
        equip_slot: ApplicationProfileDecorationEquipSlot,
    ) -> Result<VrchatApiResponse> {
        let detail = format!("Equipping profile decoration {inventory_id}.");
        let request = inventory_item_equip_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            inventory_id,
            profile_decoration_equip_slot(equip_slot),
        )?;
        self.execute(
            "app__vrchat_media_profile_decoration_equip",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn unequip_profile_decoration(
        &self,
        equip_slot: ApplicationProfileDecorationEquipSlot,
    ) -> Result<VrchatApiResponse> {
        let equip_slot = profile_decoration_equip_slot(equip_slot);
        let detail = format!(
            "Unequipping profile decoration slot {}.",
            equip_slot.as_str()
        );
        let request = inventory_slot_unequip_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), equip_slot)?;
        self.execute(
            "app__vrchat_media_profile_decoration_unequip",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn user_inventory_item(
        &self,
        user_id: String,
        inventory_id: String,
    ) -> Result<VrchatApiResponse> {
        let detail = format!("Getting inventory item {inventory_id}.");
        let request = user_inventory_item_get_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            user_id,
            inventory_id,
        )?;
        self.execute(
            "app__vrchat_media_user_inventory_item_get",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn update_inventory_item(
        &self,
        inventory_id: String,
        params: ApplicationInventoryItemUpdateRequest,
    ) -> Result<VrchatApiResponse> {
        let detail = format!("Updating inventory item {inventory_id}.");
        let request = inventory_item_update_input(
            VRCHAT_API_DEFAULT_ENDPOINT.into(),
            inventory_id,
            InventoryItemUpdateRequest {
                is_archived: params.is_archived,
            },
        )?;
        self.execute(
            "app__vrchat_media_inventory_item_update",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn consume_inventory_bundle(
        &self,
        inventory_id: String,
    ) -> Result<VrchatApiResponse> {
        let detail = format!("Consuming inventory bundle {inventory_id}.");
        let request =
            inventory_bundle_consume_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), inventory_id)?;
        self.execute(
            "app__vrchat_media_inventory_bundle_consume",
            detail,
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    pub async fn redeem_reward(&self, code: String) -> Result<VrchatApiResponse> {
        let request = reward_redeem_input(VRCHAT_API_DEFAULT_ENDPOINT.into(), code)?;
        self.execute(
            "app__vrchat_media_reward_redeem",
            "Redeeming reward.",
            request,
            VrchatScope::VrchatMedia,
        )
        .await
    }

    async fn execute_media_upload(
        &self,
        command: &str,
        detail: impl Into<String>,
        request: VrchatApiRequest,
    ) -> Result<VrchatApiResponse> {
        let request = self.media.prepare_media_upload_request(request)?;
        self.execute(command, detail, request, VrchatScope::VrchatMedia)
            .await
    }

    async fn execute(
        &self,
        command: &str,
        detail: impl Into<String>,
        request: VrchatApiRequest,
        scope: VrchatScope,
    ) -> Result<VrchatApiResponse> {
        self.api.execute(command, detail, request, scope).await
    }
}

fn avatar_list_sort(value: ApplicationAvatarListSort) -> AvatarListSort {
    match value {
        ApplicationAvatarListSort::Created => AvatarListSort::Created,
        ApplicationAvatarListSort::Updated => AvatarListSort::Updated,
        ApplicationAvatarListSort::Order => AvatarListSort::Order,
        ApplicationAvatarListSort::CreatedAt => AvatarListSort::CreatedAt,
        ApplicationAvatarListSort::UpdatedAt => AvatarListSort::UpdatedAt,
    }
}

fn query_order(value: ApplicationQueryOrder) -> QueryOrder {
    match value {
        ApplicationQueryOrder::Ascending => QueryOrder::Ascending,
        ApplicationQueryOrder::Descending => QueryOrder::Descending,
    }
}

fn release_status_filter(value: ApplicationReleaseStatusFilter) -> ReleaseStatusFilter {
    match value {
        ApplicationReleaseStatusFilter::All => ReleaseStatusFilter::All,
        ApplicationReleaseStatusFilter::Hidden => ReleaseStatusFilter::Hidden,
        ApplicationReleaseStatusFilter::Private => ReleaseStatusFilter::Private,
        ApplicationReleaseStatusFilter::Public => ReleaseStatusFilter::Public,
    }
}

fn world_search_sort(value: ApplicationWorldSearchSort) -> WorldSearchSort {
    match value {
        ApplicationWorldSearchSort::CreatedAt => WorldSearchSort::CreatedAt,
        ApplicationWorldSearchSort::UpdatedAt => WorldSearchSort::UpdatedAt,
        ApplicationWorldSearchSort::Created => WorldSearchSort::Created,
        ApplicationWorldSearchSort::Favorites => WorldSearchSort::Favorites,
        ApplicationWorldSearchSort::Heat => WorldSearchSort::Heat,
        ApplicationWorldSearchSort::LabsPublicationDate => WorldSearchSort::LabsPublicationDate,
        ApplicationWorldSearchSort::Magic => WorldSearchSort::Magic,
        ApplicationWorldSearchSort::Name => WorldSearchSort::Name,
        ApplicationWorldSearchSort::Order => WorldSearchSort::Order,
        ApplicationWorldSearchSort::Popularity => WorldSearchSort::Popularity,
        ApplicationWorldSearchSort::PublicationDate => WorldSearchSort::PublicationDate,
        ApplicationWorldSearchSort::Random => WorldSearchSort::Random,
        ApplicationWorldSearchSort::Relevance => WorldSearchSort::Relevance,
        ApplicationWorldSearchSort::ReportCount => WorldSearchSort::ReportCount,
        ApplicationWorldSearchSort::ReportScore => WorldSearchSort::ReportScore,
        ApplicationWorldSearchSort::Shuffle => WorldSearchSort::Shuffle,
        ApplicationWorldSearchSort::Trust => WorldSearchSort::Trust,
        ApplicationWorldSearchSort::Updated => WorldSearchSort::Updated,
    }
}

fn user_search_custom_field(value: ApplicationUserSearchCustomField) -> UserSearchCustomField {
    match value {
        ApplicationUserSearchCustomField::Bio => UserSearchCustomField::Bio,
        ApplicationUserSearchCustomField::DisplayName => UserSearchCustomField::DisplayName,
    }
}

fn user_search_sort(value: ApplicationUserSearchSort) -> UserSearchSort {
    match value {
        ApplicationUserSearchSort::CreatedAt => UserSearchSort::CreatedAt,
        ApplicationUserSearchSort::Created => UserSearchSort::Created,
        ApplicationUserSearchSort::LastLogin => UserSearchSort::LastLogin,
        ApplicationUserSearchSort::NuisanceFactor => UserSearchSort::NuisanceFactor,
        ApplicationUserSearchSort::Relevance => UserSearchSort::Relevance,
    }
}

fn world_search_params(value: ApplicationWorldSearchParams) -> WorldSearchParams {
    WorldSearchParams {
        featured: value.featured,
        sort: value.sort.map(world_search_sort),
        user: value.user,
        user_id: value.user_id,
        n: value.n,
        order: value.order.map(query_order),
        offset: value.offset,
        search: value.search,
        tag: value.tag,
        notag: value.notag,
        release_status: value.release_status.map(release_status_filter),
        max_unity_version: value.max_unity_version,
        min_unity_version: value.min_unity_version,
        platform: value.platform,
        noplatform: value.noplatform,
        fuzzy: value.fuzzy,
        avatar_specific: value.avatar_specific,
    }
}

fn user_search_params(value: ApplicationUserSearchParams) -> UserSearchParams {
    UserSearchParams {
        search: value.search,
        developer_type: value.developer_type,
        n: value.n,
        offset: value.offset,
        is_internal_variant: value.is_internal_variant,
        custom_fields: value.custom_fields.map(user_search_custom_field),
        sort: value.sort.map(user_search_sort),
        order: value.order.map(query_order),
    }
}

fn group_search_params(value: ApplicationGroupSearchParams) -> GroupSearchParams {
    GroupSearchParams {
        query: value.query,
        offset: value.offset,
        n: value.n,
    }
}

fn instance_create_request(value: ApplicationInstanceCreateRequest) -> InstanceCreateRequest {
    InstanceCreateRequest {
        r#type: match value.r#type {
            ApplicationInstanceCreateType::Friends => InstanceCreateType::Friends,
            ApplicationInstanceCreateType::Group => InstanceCreateType::Group,
            ApplicationInstanceCreateType::Hidden => InstanceCreateType::Hidden,
            ApplicationInstanceCreateType::Private => InstanceCreateType::Private,
            ApplicationInstanceCreateType::Public => InstanceCreateType::Public,
        },
        can_request_invite: value.can_request_invite,
        world_id: value.world_id,
        owner_id: value.owner_id,
        region: match value.region {
            ApplicationInstanceCreateRegion::Eu => InstanceCreateRegion::Eu,
            ApplicationInstanceCreateRegion::Jp => InstanceCreateRegion::Jp,
            ApplicationInstanceCreateRegion::Us => InstanceCreateRegion::Us,
            ApplicationInstanceCreateRegion::Use => InstanceCreateRegion::Use,
        },
        group_access_type: value.group_access_type.map(|value| match value {
            ApplicationInstanceCreateGroupAccessType::Members => {
                InstanceCreateGroupAccessType::Members
            }
            ApplicationInstanceCreateGroupAccessType::Plus => InstanceCreateGroupAccessType::Plus,
            ApplicationInstanceCreateGroupAccessType::Public => {
                InstanceCreateGroupAccessType::Public
            }
        }),
        queue_enabled: value.queue_enabled,
        role_ids: value.role_ids,
        age_gate: value.age_gate,
        display_name: value.display_name,
        minimum_avatar_performance: value.minimum_avatar_performance.map(|value| match value {
            ApplicationInstanceCreateMinimumAvatarPerformance::Poor => {
                InstanceCreateMinimumAvatarPerformance::Poor
            }
            ApplicationInstanceCreateMinimumAvatarPerformance::Medium => {
                InstanceCreateMinimumAvatarPerformance::Medium
            }
            ApplicationInstanceCreateMinimumAvatarPerformance::Good => {
                InstanceCreateMinimumAvatarPerformance::Good
            }
        }),
    }
}

fn calendar_list_params(value: ApplicationCalendarListParams) -> CalendarListParams {
    CalendarListParams {
        n: value.n,
        offset: value.offset,
        date: value.date,
    }
}

fn invite_message_type(value: ApplicationInviteMessageType) -> InviteMessageType {
    match value {
        ApplicationInviteMessageType::Message => InviteMessageType::Message,
        ApplicationInviteMessageType::Request => InviteMessageType::Request,
        ApplicationInviteMessageType::RequestResponse => InviteMessageType::RequestResponse,
        ApplicationInviteMessageType::Response => InviteMessageType::Response,
    }
}

fn request_invite_request(value: ApplicationRequestInviteRequest) -> RequestInviteRequest {
    RequestInviteRequest {
        request_slot: value.request_slot,
    }
}

fn media_file_list_params(value: ApplicationMediaFileListParams) -> MediaFileListParams {
    MediaFileListParams {
        n: value.n,
        offset: value.offset,
        tag: value.tag.map(|value| match value {
            ApplicationMediaFileTag::Gallery => MediaFileTag::Gallery,
            ApplicationMediaFileTag::AvatarGallery => MediaFileTag::AvatarGallery,
            ApplicationMediaFileTag::Icon => MediaFileTag::Icon,
            ApplicationMediaFileTag::Emoji => MediaFileTag::Emoji,
            ApplicationMediaFileTag::EmojiAnimated => MediaFileTag::EmojiAnimated,
            ApplicationMediaFileTag::Sticker => MediaFileTag::Sticker,
        }),
    }
}

fn image_animation_style(value: ApplicationImageAnimationStyle) -> ImageAnimationStyle {
    match value {
        ApplicationImageAnimationStyle::Aura => ImageAnimationStyle::Aura,
        ApplicationImageAnimationStyle::Bats => ImageAnimationStyle::Bats,
        ApplicationImageAnimationStyle::Bees => ImageAnimationStyle::Bees,
        ApplicationImageAnimationStyle::Bounce => ImageAnimationStyle::Bounce,
        ApplicationImageAnimationStyle::Cloud => ImageAnimationStyle::Cloud,
        ApplicationImageAnimationStyle::Confetti => ImageAnimationStyle::Confetti,
        ApplicationImageAnimationStyle::Crying => ImageAnimationStyle::Crying,
        ApplicationImageAnimationStyle::Dislike => ImageAnimationStyle::Dislike,
        ApplicationImageAnimationStyle::Fire => ImageAnimationStyle::Fire,
        ApplicationImageAnimationStyle::Idea => ImageAnimationStyle::Idea,
        ApplicationImageAnimationStyle::Lasers => ImageAnimationStyle::Lasers,
        ApplicationImageAnimationStyle::Like => ImageAnimationStyle::Like,
        ApplicationImageAnimationStyle::Magnet => ImageAnimationStyle::Magnet,
        ApplicationImageAnimationStyle::Mistletoe => ImageAnimationStyle::Mistletoe,
        ApplicationImageAnimationStyle::Money => ImageAnimationStyle::Money,
        ApplicationImageAnimationStyle::Noise => ImageAnimationStyle::Noise,
        ApplicationImageAnimationStyle::Orbit => ImageAnimationStyle::Orbit,
        ApplicationImageAnimationStyle::Pizza => ImageAnimationStyle::Pizza,
        ApplicationImageAnimationStyle::Rain => ImageAnimationStyle::Rain,
        ApplicationImageAnimationStyle::Rotate => ImageAnimationStyle::Rotate,
        ApplicationImageAnimationStyle::Shake => ImageAnimationStyle::Shake,
        ApplicationImageAnimationStyle::Snow => ImageAnimationStyle::Snow,
        ApplicationImageAnimationStyle::Snowball => ImageAnimationStyle::Snowball,
        ApplicationImageAnimationStyle::Spin => ImageAnimationStyle::Spin,
        ApplicationImageAnimationStyle::Splash => ImageAnimationStyle::Splash,
        ApplicationImageAnimationStyle::Stop => ImageAnimationStyle::Stop,
        ApplicationImageAnimationStyle::Zzz => ImageAnimationStyle::Zzz,
    }
}

fn emoji_upload_params(value: ApplicationEmojiUploadParams) -> EmojiUploadParams {
    match value {
        ApplicationEmojiUploadParams::Emoji {
            animation_style,
            mask_tag,
        } => EmojiUploadParams::Emoji {
            animation_style: image_animation_style(animation_style),
            mask_tag: match mask_tag {
                ApplicationImageMaskTag::Square => ImageMaskTag::Square,
            },
        },
        ApplicationEmojiUploadParams::EmojiAnimated {
            animation_style,
            mask_tag,
            frames,
            frames_over_time,
            loop_style,
        } => EmojiUploadParams::EmojiAnimated {
            animation_style: image_animation_style(animation_style),
            mask_tag: match mask_tag {
                ApplicationImageMaskTag::Square => ImageMaskTag::Square,
            },
            frames,
            frames_over_time,
            loop_style: loop_style.map(|value| match value {
                ApplicationEmojiLoopStyle::PingPong => EmojiLoopStyle::PingPong,
            }),
        },
    }
}

fn print_upload_params(value: ApplicationPrintUploadParams) -> PrintUploadParams {
    PrintUploadParams {
        note: value.note,
        timestamp: value.timestamp,
    }
}

fn inventory_list_params(value: ApplicationInventoryListParams) -> InventoryListParams {
    InventoryListParams {
        n: value.n,
        offset: value.offset,
        holder_id: value.holder_id,
        equip_slot: value.equip_slot,
        order: value.order.map(|value| match value {
            ApplicationInventoryOrder::Newest => InventoryOrder::Newest,
        }),
        tags: value.tags,
        types: value.types,
        flags: value.flags,
        not_types: value.not_types,
        not_flags: value.not_flags,
        archived: value.archived,
    }
}

fn profile_decoration_equip_slot(
    value: ApplicationProfileDecorationEquipSlot,
) -> ProfileDecorationEquipSlot {
    match value {
        ApplicationProfileDecorationEquipSlot::IconFrame => ProfileDecorationEquipSlot::IconFrame,
        ApplicationProfileDecorationEquipSlot::ProfileEffect => {
            ProfileDecorationEquipSlot::ProfileEffect
        }
        ApplicationProfileDecorationEquipSlot::NameplateEffect => {
            ProfileDecorationEquipSlot::NameplateEffect
        }
    }
}

fn media_asset_upload_request(
    value: ApplicationMediaAssetUploadRequest,
) -> MediaAssetUploadRequest {
    match value {
        ApplicationMediaAssetUploadRequest::Gallery { image_data } => {
            MediaAssetUploadRequest::Gallery { image_data }
        }
        ApplicationMediaAssetUploadRequest::Icons { image_data } => {
            MediaAssetUploadRequest::Icons { image_data }
        }
        ApplicationMediaAssetUploadRequest::Emojis { image_data, params } => {
            MediaAssetUploadRequest::Emojis {
                image_data,
                params: emoji_upload_params(params),
            }
        }
        ApplicationMediaAssetUploadRequest::Stickers { image_data } => {
            MediaAssetUploadRequest::Stickers { image_data }
        }
        ApplicationMediaAssetUploadRequest::Prints {
            image_data,
            crop_white_border,
            params,
        } => MediaAssetUploadRequest::Prints {
            image_data,
            crop_white_border,
            params: print_upload_params(params),
        },
    }
}
