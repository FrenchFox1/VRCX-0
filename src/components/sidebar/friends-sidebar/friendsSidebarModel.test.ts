import { describe, expect, it } from 'vitest';

import {
    buildSameInstanceGroups,
    readFriendRefLocation,
    readFriendStatusSource,
    resolveCurrentUserStateBucket,
    resolveSidebarStatusDotClassName,
    toLegacyFriendSortRow
} from './friendsSidebarModel';

describe('friendsSidebarModel same-instance groups', () => {
    it('groups one friend with the current user but not a solo friend elsewhere', () => {
        const currentLocation = 'wrld_aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:123';
        const otherLocation = 'wrld_bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb:456';
        const friendWithCurrentUser = {
            id: 'usr_1',
            displayName: 'With current user',
            state: 'online',
            location: currentLocation,
            $location_at: 1
        };
        const soloElsewhere = {
            id: 'usr_2',
            displayName: 'Solo elsewhere',
            state: 'online',
            location: otherLocation,
            $location_at: 1
        };

        expect(
            buildSameInstanceGroups(
                [friendWithCurrentUser, soloElsewhere],
                { isShowCurrentUserInSameInstance: true },
                { location: currentLocation }
            )
        ).toEqual([
            {
                location: currentLocation,
                rows: [friendWithCurrentUser],
                isCurrentInstance: true
            }
        ]);
    });

    it('requires two friends in the current instance when the current user is hidden', () => {
        const currentLocation = 'wrld_current:123';
        const friend = {
            id: 'usr_friend',
            displayName: 'Friend',
            state: 'online',
            location: currentLocation,
            $location_at: 1
        };

        expect(
            buildSameInstanceGroups(
                [friend],
                { isShowCurrentUserInSameInstance: false },
                { location: currentLocation }
            )
        ).toEqual([]);
    });
});

describe('friendsSidebarModel friend status source', () => {
    it('uses top-level roster presence over stale nested ref presence', () => {
        const friend = {
            id: 'usr_friend',
            displayName: 'Friend',
            state: 'online',
            location: 'wrld_live:123',
            status: 'join me',
            ref: {
                id: 'usr_friend',
                displayName: 'Friend',
                state: 'offline',
                location: 'offline',
                status: 'active'
            }
        };

        const source = readFriendStatusSource(friend);
        const sortRow = toLegacyFriendSortRow(friend);

        expect(source).toMatchObject({
            state: 'online',
            location: 'wrld_live:123',
            status: 'join me'
        });
        expect(readFriendRefLocation(friend)).toBe('wrld_live:123');
        expect(sortRow.ref).toMatchObject({
            state: 'online',
            location: 'wrld_live:123',
            status: 'join me'
        });
    });
});

describe('friendsSidebarModel current user status dot', () => {
    const currentUser = {
        id: 'usr_self',
        status: 'active',
        state: 'online'
    };

    it('defaults to the active outline when local game state is unavailable', () => {
        expect(
            resolveSidebarStatusDotClassName(currentUser, currentUser, true)
        ).toBe(
            'user-status-indicator online border-[var(--status-online)] bg-background'
        );
    });

    it('uses the solid status colour while the local game is running', () => {
        expect(
            resolveSidebarStatusDotClassName(currentUser, currentUser, true, {
                isGameRunning: true
            })
        ).toBe('user-status-indicator online bg-[var(--status-online)]');
    });

    it('keeps the logged-in current user active when the local game is stopped', () => {
        const stoppedCurrentUser = {
            id: 'usr_self',
            status: 'busy',
            state: 'offline',
            location: 'offline'
        };

        expect(
            resolveSidebarStatusDotClassName(
                stoppedCurrentUser,
                stoppedCurrentUser,
                true,
                { isGameRunning: false }
            )
        ).toBe(
            'user-status-indicator busy border-[var(--status-busy)] bg-background'
        );
    });

    it('keeps local game authority above stale remote presence fields', () => {
        const runningCurrentUser = {
            id: 'usr_self',
            status: 'busy',
            state: 'offline',
            location: 'offline'
        };

        expect(
            resolveSidebarStatusDotClassName(
                runningCurrentUser,
                runningCurrentUser,
                true,
                { isGameRunning: true }
            )
        ).toBe('user-status-indicator busy bg-[var(--status-busy)]');
    });

    it('uses the solid account status when the stopped local game has a remote location', () => {
        const dialogUser = {
            id: 'usr_self',
            status: 'active',
            state: 'offline',
            location: 'offline'
        };
        const currentUserSnapshot = {
            id: 'usr_self',
            status: 'busy',
            state: 'online',
            location: 'wrld_remote:456'
        };

        expect(
            resolveSidebarStatusDotClassName(
                dialogUser,
                currentUserSnapshot,
                true,
                { isGameRunning: false }
            )
        ).toBe('user-status-indicator busy bg-[var(--status-busy)]');
    });

    it('uses the account status color for remote play', () => {
        const remoteCurrentUser = {
            id: 'usr_self',
            status: 'join me',
            state: 'online',
            location: 'wrld_remote:456'
        };

        expect(
            resolveSidebarStatusDotClassName(
                remoteCurrentUser,
                remoteCurrentUser,
                true,
                { isGameRunning: false }
            )
        ).toBe('user-status-indicator joinme bg-[var(--status-joinme)]');
    });
});

describe('friendsSidebarModel current user state bucket', () => {
    it('ignores remote online state when there is no location', () => {
        expect(
            resolveCurrentUserStateBucket({
                id: 'usr_self',
                state: 'online',
                location: ''
            })
        ).toBe('active');
    });

    it('uses active instead of offline after login without a location', () => {
        expect(
            resolveCurrentUserStateBucket({
                id: 'usr_self',
                state: 'offline',
                location: 'offline'
            })
        ).toBe('active');
    });

    it('uses online when a remote location contradicts embedded offline state', () => {
        expect(
            resolveCurrentUserStateBucket({
                id: 'usr_self',
                state: 'offline',
                location: 'wrld_remote:456'
            })
        ).toBe('online');
    });
});

describe('friendsSidebarModel ordinary friend status dot', () => {
    const currentUser = { id: 'usr_self' };

    it('does not let the local game flag change an ordinary online friend', () => {
        const friend = {
            id: 'usr_friend',
            status: 'busy',
            state: 'online',
            location: 'wrld_friend:123'
        };

        expect(
            resolveSidebarStatusDotClassName(friend, currentUser, false, {
                isGameRunning: false
            })
        ).toBe('user-status-indicator busy bg-[var(--status-busy)]');
        expect(
            resolveSidebarStatusDotClassName(friend, currentUser, false, {
                isGameRunning: true
            })
        ).toBe('user-status-indicator busy bg-[var(--status-busy)]');
    });

    it('keeps an ordinary pending friend offline', () => {
        const friend = {
            id: 'usr_friend',
            status: 'join me',
            state: 'online',
            location: 'wrld_friend:123',
            pendingOffline: true
        };

        expect(
            resolveSidebarStatusDotClassName(friend, currentUser, false, {
                isGameRunning: false
            })
        ).toBe('user-status-indicator offline bg-[var(--status-offline)]');
    });
});
