import { describe, expect, it } from 'vitest';

import {
    buildGameLogSessionDurationDetails,
    getGameLogSessionPlayerDuration
} from './gameLogSessionDurations';

describe('gameLogSessionDurations', () => {
    it('preserves id and display-name accumulation when rows are batched', () => {
        const details = buildGameLogSessionDurationDetails([
            {
                displayName: 'Alice',
                userId: 'usr_alice',
                time: 60_000
            },
            {
                display_name: 'Renamed Alice',
                user_id: 'usr_alice',
                time: 90_000
            },
            {
                displayName: 'Fallback User',
                time: 20_000
            },
            {
                display_name: 'fallback user',
                time: 30_000
            },
            {
                displayName: 'Ignored',
                time: 0
            },
            {
                displayName: 'Ignored',
                time: -1
            }
        ]);

        expect(details.maxDurationMs).toBe(150_000);
        expect(
            getGameLogSessionPlayerDuration(details.durationByKey, {
                displayName: 'Different Name',
                userId: 'usr_alice'
            })
        ).toBe(150_000);
        expect(
            getGameLogSessionPlayerDuration(details.durationByKey, {
                displayName: 'FALLBACK USER'
            })
        ).toBe(50_000);
    });
});
