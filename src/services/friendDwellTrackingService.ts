import type {
    FriendRecord,
    FriendRosterById,
    FriendRosterStore
} from '@/domain/friends/types';
import { normalizeStateBucket } from '@/domain/users/userFacts';
import { timestampMsFromValue } from '@/shared/utils/dateTime';
import { parseLocation } from '@/shared/utils/location';
import { isRecord } from '@/shared/utils/record';
import { normalizeString as normalizeId } from '@/shared/utils/string';
import { useFriendRosterStore } from '@/state/friendRosterStore';

const firstSeenByUser = new Map<string, { location: string; since: number }>();
let started = false;
let previousFriendsById: FriendRosterById | null = null;

function getFriendRefRecord(friend: FriendRecord): Record<string, unknown> {
    return isRecord(friend.ref) ? friend.ref : friend;
}

function readLocationProjectionTag(value: unknown): unknown {
    return isRecord(value) ? value.tag : undefined;
}

function readEntryLocationTag(friend: FriendRecord) {
    const ref = getFriendRefRecord(friend);
    return normalizeId(
        friend.location ||
            ref.location ||
            friend.$location?.tag ||
            readLocationProjectionTag(ref.$location)
    );
}

function readEntryUpstreamEpoch(friend: FriendRecord) {
    const ref = getFriendRefRecord(friend);
    return timestampMsFromValue(
        friend.locationAt ||
            ref.locationAt ||
            friend.$location_at ||
            ref.$location_at
    );
}

function applyFriendChange(userId: string, friend: FriendRecord) {
    const stateBucket = normalizeStateBucket(friend.state);
    const locationTag = readEntryLocationTag(friend);
    const inRealInstance =
        stateBucket === 'online' && parseLocation(locationTag).isRealInstance;

    if (!inRealInstance || readEntryUpstreamEpoch(friend)) {
        firstSeenByUser.delete(userId);
        return;
    }

    const tracked = firstSeenByUser.get(userId);
    if (!tracked || tracked.location !== locationTag) {
        firstSeenByUser.set(userId, {
            location: locationTag,
            since: Date.now()
        });
    }
}

function ingestRosterState(state: FriendRosterStore) {
    const friendsById = state.friendsById;
    if (friendsById === previousFriendsById) {
        return;
    }
    const previous = previousFriendsById || {};
    previousFriendsById = friendsById;

    for (const userId in previous) {
        if (!friendsById[userId]) {
            firstSeenByUser.delete(userId);
        }
    }

    for (const userId in friendsById) {
        const friend = friendsById[userId];
        if (friend === previous[userId]) {
            continue;
        }
        applyFriendChange(normalizeId(friend.id || userId), friend);
    }
}

function ensureStarted() {
    if (started) {
        return;
    }
    started = true;
    ingestRosterState(useFriendRosterStore.getState());
    useFriendRosterStore.subscribe(ingestRosterState);
}

export function resetFriendDwellTracking() {
    firstSeenByUser.clear();
    previousFriendsById = null;
}

export function getEstimatedDwellSince(userId: string, location: string) {
    ensureStarted();
    const tracked = firstSeenByUser.get(normalizeId(userId));
    if (tracked && tracked.location === normalizeId(location)) {
        return tracked.since;
    }
    return 0;
}

ensureStarted();
