import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    getCurrentInstanceSnapshot: vi.fn()
}));

vi.mock('@/repositories/currentInstanceRosterRepository', () => ({
    default: {
        getCurrentInstanceSnapshot: mocks.getCurrentInstanceSnapshot
    }
}));

import { loadCurrentInstanceRoster } from './currentInstanceRosterService';

const runtimePlayer = {
    id: 'usr_runtime',
    userId: 'usr_runtime',
    displayName: 'Runtime Player',
    joinedAt: '2026-08-01T01:00:00.000Z',
    joinedAtMs: Date.parse('2026-08-01T01:00:00.000Z'),
    lastDurationMs: 0,
    source: 'runtime' as const
};

const worldId = 'wrld_00000000-0000-0000-0000-000000000000';

describe('currentInstanceRosterService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('reads the authoritative backend roster for current-instance dialogs', async () => {
        mocks.getCurrentInstanceSnapshot.mockResolvedValueOnce({
            context: {
                createdAt: '2026-08-01T01:00:00.000Z',
                groupName: '',
                location: `${worldId}:1~region(jp)`,
                playerCount: 1,
                playerFactsKnown: true,
                source: 'runtime',
                time: 0,
                worldId,
                worldName: 'Runtime World'
            },
            players: [runtimePlayer]
        });
        await expect(
            loadCurrentInstanceRoster({
                currentLocation: `${worldId}:1~region(jp)`
            })
        ).resolves.toEqual({
            context: {
                createdAt: '2026-08-01T01:00:00.000Z',
                groupName: '',
                location: `${worldId}:1~region(jp)`,
                playerCount: 1,
                playerFactsKnown: true,
                source: 'runtime',
                time: 0,
                worldId,
                worldName: 'Runtime World'
            },
            players: [runtimePlayer]
        });
        expect(mocks.getCurrentInstanceSnapshot).toHaveBeenCalledTimes(1);
    });

    it('preserves the backend response without inventing another roster', async () => {
        mocks.getCurrentInstanceSnapshot.mockResolvedValueOnce({
            context: {
                createdAt: '2026-08-01T00:00:00.000Z',
                groupName: '',
                location: `${worldId}:1~region(jp)`,
                playerCount: 1,
                source: 'runtime',
                time: 0,
                worldId,
                worldName: 'Recovered World'
            },
            players: [
                {
                    id: 'usr_recovered',
                    userId: 'usr_recovered',
                    displayName: 'Recovered Player',
                    joinedAt: '2026-08-01T00:00:00.000Z',
                    joinedAtMs: Date.parse('2026-08-01T00:00:00.000Z')
                }
            ]
        });

        const result = await loadCurrentInstanceRoster({
            currentLocation: `${worldId}:1~region(jp)`
        });

        expect(result.context.source).toBe('runtime');
        expect(result.players[0]?.displayName).toBe('Recovered Player');
        expect(mocks.getCurrentInstanceSnapshot).toHaveBeenCalledWith({
            currentLocation: `${worldId}:1~region(jp)`
        });
    });
});
