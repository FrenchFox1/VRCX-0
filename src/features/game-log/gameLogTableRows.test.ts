import { describe, expect, it } from 'vitest';

import { sortGameLogTableRows } from './gameLogTableRows';
import type { GameLogRow } from './gameLogTypes';

describe('sortGameLogTableRows', () => {
    it('keeps query order and row references when sorting is cleared or disabled', () => {
        const rows: GameLogRow[] = [
            { rowId: 2, created_at: '', type: 'VideoPlay' },
            { rowId: 1, created_at: '', type: 'Location' }
        ];

        expect(sortGameLogTableRows(rows, [])).toBe(rows);
        expect(
            sortGameLogTableRows(rows, [
                { id: 'displayName', desc: false },
                { id: 'detail', desc: true }
            ])
        ).toBe(rows);
        const sorted = sortGameLogTableRows(rows, [
            { id: 'type', desc: false }
        ]);
        expect(sorted).toEqual([rows[1], rows[0]]);
        expect(sorted[0]).toBe(rows[1]);
        expect(rows.map((row) => row.rowId)).toEqual([2, 1]);
    });

    it('sorts dates chronologically across time zones and breaks equal timestamps by row ID', () => {
        const rows: GameLogRow[] = [
            {
                rowId: 2,
                created_at: '2026-08-31T09:00:00+09:00',
                type: 'Location'
            },
            { rowId: 1, created_at: '2026-08-31T01:00:00Z', type: 'Location' },
            { rowId: 10, created_at: '2026-08-31T00:00:00Z', type: 'Location' }
        ];

        expect(
            sortGameLogTableRows(rows, [{ id: 'created_at', desc: true }]).map(
                (row) => row.rowId
            )
        ).toEqual([1, 10, 2]);
    });

    it('retains the row ID fallback when either timestamp is invalid', () => {
        const invalid: GameLogRow = {
            rowId: 10,
            created_at: '',
            type: 'Location'
        };
        const valid: GameLogRow = {
            rowId: 2,
            created_at: '2026-08-31T00:00:00Z',
            type: 'Location'
        };

        const sorting = [{ id: 'created_at', desc: false }];
        expect(sortGameLogTableRows([invalid, valid], sorting)).toEqual([
            valid,
            invalid
        ]);
        expect(sortGameLogTableRows([valid, invalid], sorting)).toEqual([
            valid,
            invalid
        ]);
        const bothInvalid = sortGameLogTableRows(
            [invalid, { ...valid, created_at: 'invalid' }],
            sorting
        );
        expect(bothInvalid.map((row) => row.rowId)).toEqual([2, 10]);
    });

    it('keeps basic string ordering, multiple sort keys, and stable ties', () => {
        const rows: GameLogRow[] = [
            { rowId: 1, created_at: '2026-08-30T00:00:00Z', type: 'Location' },
            { rowId: 2, created_at: '2026-08-31T00:00:00Z', type: 'Location' },
            {
                rowId: 2,
                created_at: '2026-08-31T00:00:00Z',
                type: 'Location',
                displayName: 'tie'
            },
            { rowId: 3, created_at: '', type: 'location' },
            { rowId: 4, created_at: '', type: 'VideoPlay' }
        ];

        const sorted = sortGameLogTableRows(rows, [
            { id: 'type', desc: false },
            { id: 'created_at', desc: true }
        ]);

        expect(sorted).toEqual([rows[1], rows[2], rows[0], rows[4], rows[3]]);
        expect(sorted[0]).toBe(rows[1]);
        expect(sorted[1]).toBe(rows[2]);
    });
});
