import type { FavoriteGroupMap } from '@/domain/favorites/types';
import { normalizeStateBucket } from '@/domain/users/userFacts';
import {
    getFriendsSortFunction,
    sortStatus,
    type FriendSortItem,
    type FriendSortMethod
} from '@/shared/utils/friend';
import { isRecord } from '@/shared/utils/record';
export { resolveCurrentInviteLocation } from '@/shared/utils/invite';
import {
    buildSameInstanceFriendGroups,
    isOnlineSameInstanceFriend,
    resolveSameInstanceFriendLocation,
    type SameInstanceLastLocation
} from '@/domain/friends/sameInstanceFriends';
import type {
    FriendProfileFields,
    FriendRecordInput
} from '@/domain/friends/types';
import { userStatusFromValue } from '@/shared/utils/friendStatus';
import {
    locationSentinel,
    normalizeLocationStatus,
    resolveFriendPresenceLocation
} from '@/shared/utils/location';
import { normalizeString as normalizeId } from '@/shared/utils/string';
import { getTrustColor, type TrustColorMap } from '@/shared/utils/trustColors';
import { computeTrustLevel } from '@/shared/utils/userTransforms';

export type SidebarFriendRecord = FriendRecordInput &
    Partial<FriendProfileFields> & {
        $friendNumber?: number;
        $lastSeen?: string | number;
        $location_at?: string | number | null;
        $online_for?: string | number;
        $userColour?: string;
        created_at?: string;
        developerType?: string;
        displayName?: string;
        id?: string;
        last_activity?: string | number;
        last_login?: string | number;
        location?: string;
        memberCount?: number;
        name?: string;
        state?: string;
        stateBucket?: string;
        tags?: string[];
        updated_at?: string;
        username?: string;
        activeFriends?: string[];
        isFriend?: boolean;
        offlineFriends?: string[];
        onlineFriends?: string[];
        pendingOffline?: boolean;
        ref?: SidebarFriendRecord | null;
        travelingToLocation?: string | null;
    };

export type SidebarPreferences = {
    isShowCurrentUserInSameInstance?: boolean;
    isHideFriendsInSameInstance?: boolean;
    isSameInstanceAboveFavorites?: boolean;
    isSidebarDivideByFriendGroup?: boolean;
    sidebarFavoriteGroupOrder?: string[];
    sidebarFavoriteGroups?: string[];
    sidebarGroupByInstance?: boolean;
    sidebarSortMethod1?: FriendSortMethod | '';
    sidebarSortMethod2?: FriendSortMethod | '';
    sidebarSortMethod3?: FriendSortMethod | '';
};

export type LastLocationSnapshot = SameInstanceLastLocation;

type SidebarStatusOptions = {
    hideNonFriend?: boolean;
    isGameRunning?: boolean | null;
};

type CurrentUserLocationSource = {
    [key: string]: unknown;
    location?: string | null;
    $location?: { tag?: string | null } | null;
};

export type SameInstanceGroup = {
    location: string;
    rows: SidebarFriendRecord[];
    isCurrentInstance: boolean;
};

function locationProjection(value: unknown): Record<string, unknown> | null {
    return isRecord(value) ? value : null;
}

function isFriendSortMethod(
    value: FriendSortMethod | '' | undefined
): value is FriendSortMethod {
    return Boolean(value);
}

export function resolvePresenceLocation(profile: unknown) {
    return resolveFriendPresenceLocation(profile);
}

export function readFriendRef(
    friend: SidebarFriendRecord | null | undefined
): SidebarFriendRecord | null | undefined {
    return friend?.ref && typeof friend.ref === 'object' ? friend.ref : friend;
}

export function readFriendStatusSource(
    friend: SidebarFriendRecord | null | undefined
) {
    const ref = readFriendRef(friend);
    if (!ref || ref === friend) {
        return friend;
    }
    return {
        ...ref,
        ...friend,
        ref,
        pendingOffline: Boolean(friend?.pendingOffline || ref?.pendingOffline)
    };
}

export function readFriendRefLocation(
    friend: SidebarFriendRecord | null | undefined
) {
    const source = readFriendStatusSource(friend);
    return normalizeId(
        source?.location || locationProjection(source?.$location)?.tag
    );
}

export function readFriendRefTravelingLocation(
    friend: SidebarFriendRecord | null | undefined
) {
    const source = readFriendStatusSource(friend);
    return normalizeId(
        source?.travelingToLocation || source?.$travelingToLocation
    );
}

export function clearStaleOfflineLocation(location: string, state: unknown) {
    const normalizedState = normalizeStateBucket(state);
    if (
        (normalizedState === 'online' || normalizedState === 'active') &&
        locationSentinel(location) === 'offline'
    ) {
        return '';
    }
    return location;
}

export function buildFavoriteIdSet(
    remoteFavoriteIds: readonly string[] | null | undefined,
    localFriendFavorites: FavoriteGroupMap | null | undefined
) {
    const ids = new Set(
        (remoteFavoriteIds || []).map(normalizeId).filter(Boolean)
    );
    for (const values of Object.values(localFriendFavorites || {})) {
        for (const id of values) {
            const normalized = normalizeId(id);
            if (normalized) {
                ids.add(normalized);
            }
        }
    }
    return ids;
}

export function resolveTrustNameColour(
    friend: SidebarFriendRecord | null | undefined,
    trustColor: TrustColorMap
) {
    if (!friend?.$trustClass && Array.isArray(friend?.tags)) {
        const trust = computeTrustLevel(
            friend.tags,
            typeof friend.developerType === 'string' ? friend.developerType : ''
        );
        return getTrustColor(
            {
                ...friend,
                $trustClass: trust.trustClass,
                $isModerator: trust.isModerator,
                $isTroll: trust.isTroll,
                $isProbableTroll: trust.isProbableTroll
            },
            trustColor
        );
    }
    return getTrustColor(friend, trustColor);
}

export function legacyStatusDotClassName(status: unknown) {
    const normalizedStatus = userStatusFromValue(status);
    if (normalizedStatus === 'active') {
        return 'bg-[var(--status-online)]';
    }
    if (normalizedStatus === 'join me') {
        return 'bg-[var(--status-joinme)]';
    }
    if (normalizedStatus === 'ask me') {
        return 'bg-[var(--status-askme)]';
    }
    if (normalizedStatus === 'busy') {
        return 'bg-[var(--status-busy)]';
    }
    return '';
}

export function resolveCurrentUserStateBucket(
    currentUser: CurrentUserLocationSource | null | undefined
) {
    const location = normalizeLocationStatus(
        currentUser?.location || locationProjection(currentUser?.$location)?.tag
    );
    if (location && locationSentinel(location) !== 'offline') {
        return 'online';
    }
    return 'active';
}

function activeStatusDotClassName(status: unknown) {
    const normalizedStatus = userStatusFromValue(status);
    if (normalizedStatus === 'join me') {
        return 'border-[var(--status-joinme)] bg-background';
    }
    if (normalizedStatus === 'ask me') {
        return 'border-[var(--status-askme)] bg-background';
    }
    if (normalizedStatus === 'busy') {
        return 'border-[var(--status-busy)] bg-background';
    }
    return 'border-[var(--status-online)] bg-background';
}

function activeStatusSortValue(friend: SidebarFriendRecord) {
    const source = readFriendStatusSource(friend);
    const normalizedStatus = userStatusFromValue(source?.status);
    if (
        normalizedStatus === 'join me' ||
        normalizedStatus === 'ask me' ||
        normalizedStatus === 'busy'
    ) {
        return normalizedStatus;
    }
    return 'active';
}

function compareByActiveStatus(
    left: SidebarFriendRecord,
    right: SidebarFriendRecord
) {
    return sortStatus(
        activeStatusSortValue(left),
        activeStatusSortValue(right)
    );
}

export function resolveSidebarStatusDotClassName(
    friend: SidebarFriendRecord | null | undefined,
    currentUser: SidebarFriendRecord | null | undefined,
    isCurrentUser = false,
    { hideNonFriend = true, isGameRunning = false }: SidebarStatusOptions = {}
) {
    const source = readFriendStatusSource(friend);
    if (!source) {
        return '';
    }
    const userId = normalizeId(source?.id || source?.userId);
    const status = userStatusFromValue(source?.status);
    const location = normalizeLocationStatus(
        source?.location || locationProjection(source?.$location)?.tag
    );
    const isOnlineByCurrentSnapshot = (
        currentUser?.onlineFriends || []
    ).includes(userId);
    const isActiveByCurrentSnapshot = (
        currentUser?.activeFriends || []
    ).includes(userId);
    const isOfflineByCurrentSnapshot = (
        currentUser?.offlineFriends || []
    ).includes(userId);
    const snapshotState = isOnlineByCurrentSnapshot
        ? 'online'
        : isActiveByCurrentSnapshot
          ? 'active'
          : isOfflineByCurrentSnapshot
            ? 'offline'
            : '';
    const state = normalizeStateBucket(source?.state || snapshotState);
    const stateBucket = state;

    if (isCurrentUser || userId === currentUser?.id) {
        const currentSource = readFriendStatusSource(currentUser) || source;
        const currentStatus = normalizeLocationStatus(
            currentSource?.status || status
        );
        const currentLocation = normalizeLocationStatus(
            currentSource?.location ||
                locationProjection(currentSource?.$location)?.tag ||
                source?.location ||
                locationProjection(source?.$location)?.tag
        );
        if (isGameRunning === true) {
            return (
                legacyStatusDotClassName(currentStatus) ||
                'bg-[var(--status-online)]'
            );
        }
        if (currentLocation && currentLocation !== 'offline') {
            return (
                legacyStatusDotClassName(currentStatus) ||
                'bg-[var(--status-online)]'
            );
        }
        return activeStatusDotClassName(currentStatus);
    }

    if (source?.pendingOffline) {
        return 'bg-[var(--status-offline)]';
    }

    if (
        hideNonFriend &&
        source?.isFriend === false &&
        friend?.isFriend === false
    ) {
        return '';
    }

    if (state === 'offline' || stateBucket === 'offline') {
        return 'bg-[var(--status-offline)]';
    }

    if (
        status !== 'active' &&
        location === 'private' &&
        state === '' &&
        userId &&
        !isOnlineByCurrentSnapshot
    ) {
        return isActiveByCurrentSnapshot
            ? activeStatusDotClassName(status)
            : 'bg-[var(--status-offline)]';
    }
    if (state === 'active') {
        return activeStatusDotClassName(status);
    }
    if (location === 'offline' && state !== 'online') {
        return 'bg-[var(--status-offline)]';
    }
    if (status === 'active') {
        return 'bg-[var(--status-online)]';
    }
    if (status === 'join me') {
        return 'bg-[var(--status-joinme)]';
    }
    if (status === 'ask me') {
        return 'bg-[var(--status-askme)]';
    }
    if (status === 'busy') {
        return 'bg-[var(--status-busy)]';
    }
    return '';
}

export function toLegacyFriendSortRow(
    friend: SidebarFriendRecord
): FriendSortItem {
    const ref = readFriendRef(friend);
    return {
        ...friend,
        name:
            friend?.name ||
            friend?.displayName ||
            friend?.username ||
            friend?.id ||
            '',
        ref: ref && ref !== friend ? { ...ref, ...friend } : friend
    } as FriendSortItem;
}

export function sortRows(
    rows: readonly SidebarFriendRecord[],
    prefs: SidebarPreferences
) {
    const methods = [
        prefs.sidebarSortMethod1,
        prefs.sidebarSortMethod2,
        prefs.sidebarSortMethod3
    ].filter(isFriendSortMethod);
    if (!methods.length) {
        return rows;
    }
    const sort = getFriendsSortFunction(methods);
    return [...rows].sort((left, right) =>
        sort(toLegacyFriendSortRow(left), toLegacyFriendSortRow(right))
    );
}

export function sortActiveRows(
    rows: readonly SidebarFriendRecord[],
    prefs: SidebarPreferences
) {
    const sortedRows = sortRows(rows, prefs);
    return [...sortedRows].sort(compareByActiveStatus);
}

export function sameInstanceLocationTag(
    friend: SidebarFriendRecord,
    lastLocation: LastLocationSnapshot | null | undefined
) {
    const source = readFriendStatusSource(friend);
    if (!isOnlineSameInstanceFriend(source)) {
        return '';
    }
    return resolveSameInstanceFriendLocation(source, lastLocation);
}

export function buildSameInstanceGroups(
    rows: readonly SidebarFriendRecord[],
    prefs: SidebarPreferences,
    lastLocation: LastLocationSnapshot | null | undefined
) {
    return buildSameInstanceFriendGroups(sortRows(rows, prefs), lastLocation, {
        includeCurrentUser: prefs.isShowCurrentUserInSameInstance !== false
    }).map(({ location, friends, isCurrentInstance }): SameInstanceGroup => ({
        location,
        rows: friends,
        isCurrentInstance
    }));
}
