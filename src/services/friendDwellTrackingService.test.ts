import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { useFriendRosterStore } from '@/state/friendRosterStore';

import {
    getEstimatedDwellSince,
    resetFriendDwellTracking
} from './friendDwellTrackingService';

describe('friendDwellTrackingService', () => {
    beforeEach(() => {
        vi.useFakeTimers();
        useFriendRosterStore.getState().resetRoster();
        resetFriendDwellTracking();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('restarts a fallback dwell estimate after a friend leaves the roster', () => {
        vi.setSystemTime(1_000);
        useFriendRosterStore.getState().setRosterSnapshot({
            currentUserId: 'usr_self',
            friendsById: {
                usr_friend: {
                    id: 'usr_friend',
                    displayName: 'Friend',
                    location: 'wrld_test:1',
                    state: 'online'
                }
            }
        });

        expect(getEstimatedDwellSince('usr_friend', 'wrld_test:1')).toBe(1_000);

        useFriendRosterStore.getState().removeFriend('usr_friend');
        vi.setSystemTime(5_000);
        useFriendRosterStore.getState().applyFriendPatch({
            userId: 'usr_friend',
            patch: {
                displayName: 'Friend',
                location: 'wrld_test:1',
                state: 'online'
            }
        });

        expect(getEstimatedDwellSince('usr_friend', 'wrld_test:1')).toBe(5_000);
    });
});
