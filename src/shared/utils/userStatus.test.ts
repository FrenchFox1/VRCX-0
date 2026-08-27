import { describe, expect, it } from 'vitest';

import { resolveUserPresenceStatus, userStatusSortRank } from './userStatus';

describe('userStatus', () => {
    it('normalizes legacy compact status strings', () => {
        expect(resolveUserPresenceStatus('joinme')).toBe('join me');
        expect(resolveUserPresenceStatus('askme')).toBe('ask me');
        expect(resolveUserPresenceStatus('offline:offline')).toBe('offline');
        expect(resolveUserPresenceStatus('private:private')).toBe('private');
        expect(resolveUserPresenceStatus('traveling:traveling')).toBe(
            'traveling'
        );
    });

    it('treats pending offline and offline fields as offline', () => {
        expect(
            resolveUserPresenceStatus({
                pendingOffline: true,
                status: 'join me'
            })
        ).toBe('offline');
        expect(
            resolveUserPresenceStatus({ state: 'active', location: 'offline' })
        ).toBe('offline');
        expect(
            resolveUserPresenceStatus({
                ref: { state: 'online', location: 'offline:offline' }
            })
        ).toBe('offline');
    });

    it('prioritizes explicit social status before active location', () => {
        expect(
            resolveUserPresenceStatus({
                status: 'join me',
                location: 'wrld_123:1'
            })
        ).toBe('join me');
        expect(
            resolveUserPresenceStatus({
                status: 'ask me',
                location: 'wrld_123:1'
            })
        ).toBe('ask me');
        expect(
            resolveUserPresenceStatus({
                status: 'busy',
                location: 'wrld_123:1'
            })
        ).toBe('busy');
        expect(resolveUserPresenceStatus({ location: 'wrld_123:1' })).toBe(
            'active'
        );
    });

    it('keeps state active distinct from online active for presence ordering', () => {
        expect(resolveUserPresenceStatus({ state: 'active' })).toBe(
            'state-active'
        );
    });

    it('orders statuses by joinability and availability', () => {
        expect(userStatusSortRank('joinme')).toBe(0);
        expect(userStatusSortRank('active')).toBe(1);
        expect(userStatusSortRank('askme')).toBe(2);
        expect(userStatusSortRank('busy')).toBe(3);
        expect(userStatusSortRank('private')).toBe(4);
        expect(userStatusSortRank('offline')).toBe(5);
    });
});
