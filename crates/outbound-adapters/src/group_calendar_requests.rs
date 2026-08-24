use vrcx_0_application::social::{GroupCalendarPageKind, GroupCalendarRemoteRequests};
use vrcx_0_application_core::{vrchat_api::VrchatApiRequest, Result};
use vrcx_0_vrchat_client::{
    groups::profile_get_input,
    tools::{
        calendars_get_input, featured_calendars_get_input, following_calendars_get_input,
        CalendarListParams,
    },
};

pub struct VrchatGroupCalendarRemoteRequests;

impl GroupCalendarRemoteRequests for VrchatGroupCalendarRemoteRequests {
    fn page(
        &self,
        endpoint: String,
        kind: GroupCalendarPageKind,
        date: String,
        n: i32,
        offset: i32,
    ) -> Result<VrchatApiRequest> {
        let params = CalendarListParams {
            n: Some(n),
            offset: Some(offset),
            date: Some(date),
        };
        Ok(match kind {
            GroupCalendarPageKind::All => calendars_get_input(endpoint, params),
            GroupCalendarPageKind::Following => following_calendars_get_input(endpoint, params),
            GroupCalendarPageKind::Featured => featured_calendars_get_input(endpoint, params),
        })
    }

    fn group_profile(&self, endpoint: String, group_id: String) -> Result<VrchatApiRequest> {
        Ok(profile_get_input(endpoint, group_id, false)?.1)
    }
}
