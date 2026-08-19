use crate::open_string_enum::open_string_enum;

open_string_enum! {
    pub enum NotificationKind {
        AvatarReviewFailure => "avatarreview.failure",
        AvatarReviewSuccess => "avatarreview.success",
        BadgeEarned => "badge.earned",
        Boop => "boop",
        EconomyAlert => "economy.alert",
        EconomyReceivedGift => "economy.received.gift",
        EventAnnouncement => "event.announcement",
        FriendRequest => "friendRequest",
        GroupAnnouncement => "group.announcement",
        GroupEventCreated => "group.event.created",
        GroupEventStarting => "group.event.starting",
        GroupInformative => "group.informative",
        GroupInvite => "group.invite",
        GroupJoinRequest => "group.joinRequest",
        GroupPost => "group.post",
        GroupTransfer => "group.transfer",
        IgnoredFriendRequest => "ignoredFriendRequest",
        Invite => "invite",
        InviteInstanceContentGated => "invite.instance.contentGated",
        InviteResponse => "inviteResponse",
        Message => "message",
        ModerationContentRestriction => "moderation.contentrestriction",
        ModerationNotice => "moderation.notice",
        ModerationReportClosed => "moderation.report.closed",
        ModerationWarningGroup => "moderation.warning.group",
        PromoRedeem => "promo.redeem",
        RequestInvite => "requestInvite",
        RequestInviteResponse => "requestInviteResponse",
        TextAdventure => "text.adventure",
        VoteToKick => "votetokick",
        VrcPlusGift => "vrcplus.gift",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::NotificationKind;

    #[test]
    fn notification_kinds_preserve_unknown_wire_values() {
        let known: NotificationKind = serde_json::from_value(json!("boop")).unwrap();
        let unknown: NotificationKind = serde_json::from_value(json!("future.notice")).unwrap();

        assert_eq!(known, NotificationKind::Boop);
        assert_eq!(unknown.as_str(), "future.notice");
        assert_eq!(
            serde_json::to_value(unknown).unwrap(),
            json!("future.notice")
        );
    }
}
