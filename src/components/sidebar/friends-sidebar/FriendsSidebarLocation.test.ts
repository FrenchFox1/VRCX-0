import { describe, expect, it } from 'vitest';

import { resolveFriendRowLocationState } from './FriendsSidebarLocation';

describe('resolveFriendRowLocationState', () => {
    it('keeps the same-instance timer visible while offline is pending', () => {
        const state = resolveFriendRowLocationState({
            friend: {
                id: 'usr_friend',
                state: 'online',
                location: 'private',
                pendingOffline: true
            },
            isGroupByInstance: true
        });

        expect(state.groupByInstanceTimerVisible).toBe(true);
        expect(state.showLocationSubline).toBe(false);
    });

    it('hides the same-instance timer after offline is confirmed', () => {
        const state = resolveFriendRowLocationState({
            friend: {
                id: 'usr_friend',
                state: 'offline',
                location: 'private'
            },
            isGroupByInstance: true
        });

        expect(state.groupByInstanceTimerVisible).toBe(false);
    });
});
