import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useFriendRosterStore } from '@/state/friendRosterStore';
import { useUserFactsStore } from '@/state/userFactsStore';

import {
    flushRealtimeRosterUpdates,
    queueRealtimeFriendRosterUpdate,
    queueRealtimeUserFactsUpdate,
    resetRealtimeRosterUpdates
} from './realtimeRosterUpdateQueue';

function seedRoster(currentUserId: string) {
    useFriendRosterStore.getState().setRosterSnapshot({
        currentUserId,
        friendsById: {
            usr_friend: {
                id: 'usr_friend',
                displayName: 'Friend',
                state: 'online'
            }
        },
        orderedFriendIds: ['usr_friend'],
        onlineIds: ['usr_friend'],
        activeIds: [],
        offlineIds: []
    });
}

function friendPatch(displayName: string) {
    return [
        {
            userId: 'usr_friend',
            patch: { id: 'usr_friend', displayName },
            stateBucketAuthority: 'preserve' as const
        }
    ];
}

describe('realtimeRosterUpdateQueue', () => {
    beforeEach(() => {
        vi.useFakeTimers();
        useFriendRosterStore.getState().resetRoster();
        useUserFactsStore.getState().resetUserFacts();
        resetRealtimeRosterUpdates();
        seedRoster('usr_self');
    });

    afterEach(() => {
        resetRealtimeRosterUpdates();
        vi.useRealTimers();
    });

    it('applies the first update immediately and coalesces the burst that follows', () => {
        queueRealtimeFriendRosterUpdate(friendPatch('First'), false);
        expect(
            useFriendRosterStore.getState().friendsById.usr_friend.displayName
        ).toBe('First');

        queueRealtimeFriendRosterUpdate(friendPatch('Second'), false);
        queueRealtimeUserFactsUpdate([
            {
                id: 'usr_friend',
                endpoint: 'https://api.example.test',
                displayName: 'Second'
            }
        ]);
        queueRealtimeFriendRosterUpdate(friendPatch('Third'), false);
        expect(
            useFriendRosterStore.getState().friendsById.usr_friend.displayName
        ).toBe('First');

        vi.advanceTimersByTime(500);
        expect(
            useFriendRosterStore.getState().friendsById.usr_friend.displayName
        ).toBe('Third');
        expect(
            useUserFactsStore.getState().usersByKey[
                'https://api.example.test::usr_friend'
            ]
        ).toMatchObject({ displayName: 'Second' });
    });

    it('drops buffered updates when the roster owner changed', () => {
        queueRealtimeFriendRosterUpdate(friendPatch('First'), false);
        queueRealtimeFriendRosterUpdate(friendPatch('Buffered'), false);

        seedRoster('usr_other');
        flushRealtimeRosterUpdates();

        expect(
            useFriendRosterStore.getState().friendsById.usr_friend.displayName
        ).toBe('Friend');
    });
});
