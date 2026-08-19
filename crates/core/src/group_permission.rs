use crate::open_string_enum::open_string_enum;

open_string_enum! {
    pub enum GroupPermission {
        All => "*",
        AnnouncementManage => "group-announcement-manage",
        AuditView => "group-audit-view",
        BansManage => "group-bans-manage",
        CalendarManage => "group-calendar-manage",
        DataManage => "group-data-manage",
        DefaultRoleManage => "group-default-role-manage",
        GalleriesManage => "group-galleries-manage",
        InstanceAgeGatedCreate => "group-instance-age-gated-create",
        InstanceAnnouncementCreate => "group-instance-announcement-create",
        InstanceBypassAvatarPerformance => "group-instance-bypass-avatar-performance",
        InstanceCalendarLink => "group-instance-calendar-link",
        InstanceJoin => "group-instance-join",
        InstanceManage => "group-instance-manage",
        InstanceModerate => "group-instance-moderate",
        InstanceOpenCreate => "group-instance-open-create",
        InstancePlusCreate => "group-instance-plus-create",
        InstancePlusPortal => "group-instance-plus-portal",
        InstancePlusPortalUnlocked => "group-instance-plus-portal-unlocked",
        InstancePublicCreate => "group-instance-public-create",
        InstanceQueuePriority => "group-instance-queue-priority",
        InstanceRestrictedCreate => "group-instance-restricted-create",
        InvitesManage => "group-invites-manage",
        MembersManage => "group-members-manage",
        MembersRemove => "group-members-remove",
        MembersViewAll => "group-members-viewall",
        RolesAssign => "group-roles-assign",
        RolesManage => "group-roles-manage",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::GroupPermission;

    #[test]
    fn group_permissions_preserve_unknown_wire_values() {
        let known: GroupPermission = serde_json::from_value(json!("group-bans-manage")).unwrap();
        let unknown: GroupPermission =
            serde_json::from_value(json!("group-future-manage")).unwrap();

        assert_eq!(known, GroupPermission::BansManage);
        assert_eq!(unknown.as_str(), "group-future-manage");
        assert_eq!(
            serde_json::to_value(unknown).unwrap(),
            json!("group-future-manage")
        );
    }
}
