import type { FriendPatchEntry } from '@/domain/friends/types';
import type { RealtimeUserRecord } from '@/services/runtime-event-bridge/realtimeProjectionTypes';
import { useFriendLogStore } from '@/state/friendLogStore';
import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useShellStore } from '@/state/shellStore';
import { useUserFactsStore } from '@/state/userFactsStore';

const COALESCE_WINDOW_MS = 500;

let pendingPatches: FriendPatchEntry[] = [];
let pendingUsers: RealtimeUserRecord[] = [];
let pendingFriendLogChanged = false;
let pendingOwnerUserId: string | null = null;
let flushTimer: ReturnType<typeof setTimeout> | null = null;
let lastFlushAt = 0;

function rosterOwnerUserId(): string | null {
    return useFriendRosterStore.getState().currentUserId;
}

function hasPendingRosterUpdates(): boolean {
    return (
        pendingPatches.length > 0 ||
        pendingUsers.length > 0 ||
        pendingFriendLogChanged
    );
}

function clearFlushTimer(): void {
    if (flushTimer === null) {
        return;
    }
    clearTimeout(flushTimer);
    flushTimer = null;
}

function clearPendingRosterUpdates(): void {
    pendingPatches = [];
    pendingUsers = [];
    pendingFriendLogChanged = false;
    pendingOwnerUserId = null;
}

function applyRosterUpdates(
    patches: FriendPatchEntry[],
    users: RealtimeUserRecord[],
    friendLogChanged: boolean
): void {
    lastFlushAt = Date.now();
    if (users.length) {
        useUserFactsStore.getState().replaceUserFacts(users);
    }
    if (patches.length) {
        useFriendRosterStore.getState().applyFriendPatches(patches);
    }
    if (friendLogChanged) {
        useShellStore.getState().notifyMenu('friend-log');
        useFriendLogStore.getState().bumpRevision();
    }
}

function scheduleFlush(): void {
    if (flushTimer !== null) {
        return;
    }
    const elapsed = Date.now() - lastFlushAt;
    const delay = Math.max(0, COALESCE_WINDOW_MS - elapsed);
    flushTimer = setTimeout(() => {
        flushTimer = null;
        flushRealtimeRosterUpdates();
    }, delay);
}

function enqueueRosterUpdate(
    patches: FriendPatchEntry[],
    users: RealtimeUserRecord[],
    friendLogChanged: boolean
): void {
    if (!patches.length && !users.length && !friendLogChanged) {
        return;
    }
    const ownerUserId = rosterOwnerUserId();
    if (
        !hasPendingRosterUpdates() &&
        Date.now() - lastFlushAt >= COALESCE_WINDOW_MS
    ) {
        applyRosterUpdates(patches, users, friendLogChanged);
        return;
    }
    if (hasPendingRosterUpdates() && pendingOwnerUserId !== ownerUserId) {
        flushRealtimeRosterUpdates();
    }
    pendingOwnerUserId = ownerUserId;
    pendingPatches.push(...patches);
    pendingUsers.push(...users);
    pendingFriendLogChanged = pendingFriendLogChanged || friendLogChanged;
    scheduleFlush();
}

export function queueRealtimeFriendRosterUpdate(
    patches: FriendPatchEntry[],
    friendLogChanged: boolean
): void {
    enqueueRosterUpdate(patches, [], friendLogChanged);
}

export function queueRealtimeUserFactsUpdate(
    users: RealtimeUserRecord[]
): void {
    enqueueRosterUpdate([], users, false);
}

export function flushRealtimeRosterUpdates(): void {
    clearFlushTimer();
    if (!hasPendingRosterUpdates()) {
        return;
    }
    const patches = pendingPatches;
    const users = pendingUsers;
    const friendLogChanged = pendingFriendLogChanged;
    const ownerUserId = pendingOwnerUserId;
    clearPendingRosterUpdates();
    if (ownerUserId !== rosterOwnerUserId()) {
        return;
    }
    applyRosterUpdates(patches, users, friendLogChanged);
}

export function resetRealtimeRosterUpdates(): void {
    clearFlushTimer();
    clearPendingRosterUpdates();
    lastFlushAt = 0;
}
