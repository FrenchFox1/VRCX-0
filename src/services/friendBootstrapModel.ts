import type { FriendRosterInputById } from '@/domain/friends/types';
import type { FriendLogCurrentRow } from '@/repositories/friendLogRepository';
import { isRecord } from '@/shared/utils/record';

export type FriendBootstrapSnapshot = Record<string, unknown> & {
    friendsById?: FriendRosterInputById;
    orderedFriendIds?: string[];
    onlineIds?: string[];
    activeIds?: string[];
    offlineIds?: string[];
    detail?: string;
};
export type FriendStateBucket = 'online' | 'active' | 'offline';
export type FriendLogBootstrapRow = FriendLogCurrentRow & {
    user_id?: string;
    $friendNumber?: number;
    $trustLevel?: string;
};
export type FriendLogSeedRow = Partial<FriendLogBootstrapRow>;
export type CurrentUserFriendSnapshot = Record<string, unknown> & {
    id?: string;
    friends?: string[];
    offlineFriends?: string[];
    activeFriends?: string[];
    onlineFriends?: string[];
};
export type FriendBootstrapOptions = {
    userId?: string;
    endpoint?: string;
    websocket?: string;
    currentUserSnapshot?: CurrentUserFriendSnapshot | null;
    preserveLoadedState?: boolean;
};
export type FriendBootstrapResult = {
    userId: string;
    count: number;
    detail: string;
    stale: boolean;
};

export function normalizeUserId(value: unknown) {
    return typeof value === 'string'
        ? value.trim()
        : String(value ?? '').trim();
}

export { isRecord };

export function normalizeStringArray(value: unknown): string[] {
    return Array.isArray(value)
        ? value.map((entry) => normalizeUserId(entry)).filter(Boolean)
        : [];
}

export function normalizeFriendsById(value: unknown): FriendRosterInputById {
    if (!isRecord(value)) {
        return {};
    }

    const friendsById: FriendRosterInputById = {};
    for (const [userId, friend] of Object.entries(value)) {
        if (isRecord(friend)) {
            friendsById[userId] = friend;
        }
    }
    return friendsById;
}

export function getDisplayName(
    user: Record<string, unknown> | null | undefined
) {
    return (
        normalizeUserId(user?.displayName) ||
        normalizeUserId(user?.username) ||
        normalizeUserId(user?.id)
    );
}

function addStateBucketIds(
    stateById: Map<string, FriendStateBucket>,
    ids: unknown,
    state: FriendStateBucket
) {
    if (!Array.isArray(ids)) {
        return;
    }

    for (const value of ids) {
        const userId = normalizeUserId(value);
        if (!userId) {
            continue;
        }
        stateById.set(userId, state);
    }
}

export function buildFriendStateMap(
    currentUserSnapshot: CurrentUserFriendSnapshot
) {
    const stateById = new Map<string, FriendStateBucket>();
    addStateBucketIds(stateById, currentUserSnapshot?.friends, 'offline');
    addStateBucketIds(
        stateById,
        currentUserSnapshot?.offlineFriends,
        'offline'
    );
    addStateBucketIds(stateById, currentUserSnapshot?.activeFriends, 'active');
    addStateBucketIds(stateById, currentUserSnapshot?.onlineFriends, 'online');

    return stateById;
}

export function hasCompleteFriendStateSnapshot(
    currentUserSnapshot: unknown
): currentUserSnapshot is CurrentUserFriendSnapshot {
    if (!isRecord(currentUserSnapshot)) {
        return false;
    }
    return (
        Array.isArray(currentUserSnapshot.friends) &&
        Array.isArray(currentUserSnapshot.offlineFriends) &&
        Array.isArray(currentUserSnapshot.activeFriends) &&
        Array.isArray(currentUserSnapshot.onlineFriends)
    );
}

export function buildFriendLogRowsById(rows: FriendLogSeedRow[] = []) {
    const rowsById = new Map<string, FriendLogSeedRow>();
    if (!Array.isArray(rows)) {
        return rowsById;
    }

    for (const row of rows) {
        const userId = normalizeUserId(row?.userId || row?.user_id);
        if (!userId) {
            continue;
        }
        rowsById.set(userId, row);
    }
    return rowsById;
}

export function buildSeedRosterFriendsById(
    stateById: Map<string, FriendStateBucket>,
    friendLogRows: FriendLogSeedRow[] = []
) {
    const rowsById = buildFriendLogRowsById(friendLogRows);
    const friendsById: FriendRosterInputById = {};

    for (const [userId, stateBucket] of stateById.entries()) {
        const row: FriendLogSeedRow = rowsById.get(userId) ?? {};
        const trustLevel = normalizeUserId(row?.trustLevel) || 'Visitor';
        const friendNumber =
            Number.parseInt(
                String(row?.friendNumber ?? row?.$friendNumber ?? 0),
                10
            ) || 0;
        const displayName = normalizeUserId(row?.displayName) || userId;
        friendsById[userId] = {
            id: userId,
            displayName,
            username: '',
            tags: [],
            developerType: '',
            platform: 'offline',
            last_platform: '',
            location: 'offline',
            state: stateBucket,
            trustLevel,
            $trustLevel: trustLevel,
            friendNumber,
            $friendNumber: friendNumber
        };
    }

    return friendsById;
}
