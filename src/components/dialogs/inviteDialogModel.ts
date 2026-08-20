import type { FavoriteGroup, FavoriteGroupMap } from '@/domain/favorites/types';
import type {
    FriendRecordInput,
    FriendRosterInputById
} from '@/domain/friends/types';
import { isRecord } from '@/shared/utils/record';
import { normalizeString as normalizeId } from '@/shared/utils/string';

type InviteCurrentUser = FriendRecordInput | null | undefined;

type InviteFavoriteInputs = {
    favoriteFriendGroups?: readonly FavoriteGroup[];
    groupedFavoriteFriendIdsByGroupKey?: Record<string, string[]> | null;
    localFriendFavoriteGroups?: readonly string[];
    localFriendFavorites?: FavoriteGroupMap | null;
};

export function onlineFriendIdsFromGroup(
    userIds: readonly string[] | null | undefined,
    friendsById: FriendRosterInputById
) {
    return (userIds ?? []).map(normalizeId).filter((userId, index, source) => {
        const friend = friendsById[userId];
        return (
            userId &&
            source.indexOf(userId) === index &&
            friend?.state === 'online'
        );
    });
}

export function displayNameForUser(
    userId: string,
    friendsById: FriendRosterInputById,
    currentUser: InviteCurrentUser
) {
    if (normalizeId(currentUser?.id) === userId) {
        return (
            normalizeId(currentUser?.displayName) ||
            normalizeId(currentUser?.username) ||
            userId
        );
    }
    const friend = friendsById[userId];
    const ref = isRecord(friend?.ref) ? friend.ref : friend;
    return (
        normalizeId(ref?.displayName) ||
        normalizeId(ref?.username) ||
        normalizeId(friend?.name) ||
        userId
    );
}

export function pushUniqueLabel(labels: string[], label: string) {
    const normalizedLabel = normalizeId(label);
    if (normalizedLabel && !labels.includes(normalizedLabel)) {
        labels.push(normalizedLabel);
    }
}

export function filterInviteUserIds({
    selectableUserIds,
    search,
    friendsById,
    currentUser
}: {
    selectableUserIds: string[];
    search: string;
    friendsById: FriendRosterInputById;
    currentUser: InviteCurrentUser;
}) {
    const query = search.trim().toLowerCase();
    if (!query) {
        return selectableUserIds;
    }
    return selectableUserIds.filter((userId) => {
        const displayName = displayNameForUser(
            userId,
            friendsById,
            currentUser
        );
        return (
            userId.toLowerCase().includes(query) ||
            displayName.toLowerCase().includes(query)
        );
    });
}

export function sortInviteUserIdsWithSelectedFirst(
    filteredUserIds: string[],
    selectedUserIdSet: ReadonlySet<string>
) {
    return [...filteredUserIds].sort((left, right) => {
        const leftSelected = selectedUserIdSet.has(normalizeId(left));
        const rightSelected = selectedUserIdSet.has(normalizeId(right));
        if (leftSelected !== rightSelected) {
            return leftSelected ? -1 : 1;
        }
        return 0;
    });
}

export function buildFavoriteGroupLabelsByUserId({
    favoriteFriendGroups,
    groupedFavoriteFriendIdsByGroupKey,
    localFriendFavoriteGroups,
    localFriendFavorites
}: InviteFavoriteInputs) {
    const labelsByUserId: Record<string, string[]> = {};
    function addLabel(userId: string, label: string) {
        const normalizedUserId = normalizeId(userId);
        if (!normalizedUserId) {
            return;
        }
        if (!labelsByUserId[normalizedUserId]) {
            labelsByUserId[normalizedUserId] = [];
        }
        pushUniqueLabel(labelsByUserId[normalizedUserId], label);
    }

    for (const group of favoriteFriendGroups ?? []) {
        const key = normalizeId(group.key);
        const label = normalizeId(group.displayName) || key;
        for (const userId of groupedFavoriteFriendIdsByGroupKey?.[key] ?? []) {
            addLabel(userId, label);
        }
    }

    for (const groupName of localFriendFavoriteGroups ??
        Object.keys(localFriendFavorites || {})) {
        const key = normalizeId(groupName);
        for (const userId of localFriendFavorites?.[key] ?? []) {
            addLabel(userId, key);
        }
    }

    return labelsByUserId;
}

export function buildFriendsInCurrentInstanceIds({
    currentLocationPlayerIds,
    friendsById
}: {
    currentLocationPlayerIds: readonly string[];
    friendsById: FriendRosterInputById;
}) {
    const ids = new Set(currentLocationPlayerIds.map(normalizeId));
    return [...ids].filter((userId) => userId && friendsById[userId]);
}

export function buildFavoriteGroupItems({
    favoriteFriendGroups,
    groupedFavoriteFriendIdsByGroupKey,
    localFriendFavoriteGroups,
    localFriendFavorites,
    friendsById
}: InviteFavoriteInputs & { friendsById: FriendRosterInputById }) {
    const remote = (favoriteFriendGroups ?? [])
        .map((group) => {
            const key = normalizeId(group.key);
            const userIds = onlineFriendIdsFromGroup(
                groupedFavoriteFriendIdsByGroupKey?.[key],
                friendsById
            );
            return {
                key: `remote:${key}`,
                label: normalizeId(group.displayName) || key,
                userIds
            };
        })
        .filter((group) => group.key && group.userIds.length);

    const local = (localFriendFavoriteGroups ?? [])
        .map((groupName) => {
            const key = normalizeId(groupName);
            const userIds = onlineFriendIdsFromGroup(
                localFriendFavorites?.[key],
                friendsById
            );
            return {
                key: `local:${key}`,
                label: key,
                userIds
            };
        })
        .filter((group) => group.key && group.userIds.length);

    return { remote, local };
}
