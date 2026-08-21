import { describe, expect, it } from 'vitest';

import { mergeActivityTimestampsIntoProfile } from './userDialogProfileSnapshot';

describe('mergeActivityTimestampsIntoProfile', () => {
    it('adopts a real timestamp reported by the snapshot', () => {
        const profile = {
            id: 'usr_1',
            last_activity: '2026-01-01T00:00:00.000Z'
        };

        const merged = mergeActivityTimestampsIntoProfile(profile, {
            id: 'usr_1',
            last_activity: '2026-02-01T00:00:00.000Z',
            last_login: '2026-02-02T00:00:00.000Z'
        });

        expect(merged).toMatchObject({
            last_activity: '2026-02-01T00:00:00.000Z',
            last_login: '2026-02-02T00:00:00.000Z'
        });
    });

    it('keeps the previously known timestamp when the snapshot omits the field', () => {
        const profile = {
            id: 'usr_1',
            last_activity: '2026-01-01T00:00:00.000Z'
        };

        const merged = mergeActivityTimestampsIntoProfile(profile, {
            id: 'usr_1'
        });

        expect(merged).toMatchObject({
            last_activity: '2026-01-01T00:00:00.000Z'
        });
    });

    it('keeps the previously known timestamp when the snapshot reports an explicit null', () => {
        // Friend-roster snapshots are unreliable (see OptionalCompactString on the
        // Rust side): a bare `null` there just means this particular payload
        // carried no activity data, not that VRChat confirmed the value is empty.
        // Regressing a known-good timestamp back to unknown on every roster patch
        // would make the profile flicker, so an explicit null must not overwrite it.
        const profile = {
            id: 'usr_1',
            last_activity: '2026-01-01T00:00:00.000Z'
        };

        const merged = mergeActivityTimestampsIntoProfile(profile, {
            id: 'usr_1',
            last_activity: null
        });

        expect(merged).toMatchObject({
            last_activity: '2026-01-01T00:00:00.000Z'
        });
    });

    it('ignores a snapshot for a different user', () => {
        const profile = {
            id: 'usr_1',
            last_activity: '2026-01-01T00:00:00.000Z'
        };

        const merged = mergeActivityTimestampsIntoProfile(profile, {
            id: 'usr_2',
            last_activity: '2026-03-01T00:00:00.000Z'
        });

        expect(merged).toBe(profile);
    });

    it('passes through a profile untouched when there is no snapshot to merge', () => {
        const profile = {
            id: 'usr_1',
            last_activity: '2026-01-01T00:00:00.000Z'
        };

        expect(mergeActivityTimestampsIntoProfile(profile, null)).toBe(profile);
        expect(mergeActivityTimestampsIntoProfile(null, { id: 'usr_1' })).toBe(
            null
        );
    });
});
