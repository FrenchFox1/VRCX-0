import type { SortingState } from '@tanstack/react-table';

import type { GameLogRow } from './gameLogTypes';

function compareGameLogRowDates(left: GameLogRow, right: GameLogRow): number {
    const leftTs = Date.parse(String(left.created_at ?? ''));
    const rightTs = Date.parse(String(right.created_at ?? ''));
    if (
        Number.isFinite(leftTs) &&
        Number.isFinite(rightTs) &&
        leftTs !== rightTs
    ) {
        return leftTs - rightTs;
    }

    return (
        (Number.parseInt(String(left.rowId ?? 0), 10) || 0) -
        (Number.parseInt(String(right.rowId ?? 0), 10) || 0)
    );
}

export function sortGameLogTableRows(
    rows: GameLogRow[],
    sorting: SortingState
): GameLogRow[] {
    const activeSorting = sorting.filter(
        ({ id }) => id === 'created_at' || id === 'type'
    );
    if (activeSorting.length === 0) {
        return rows;
    }

    return rows.slice().sort((left, right) => {
        for (const { id, desc } of activeSorting) {
            let comparison = 0;
            if (id === 'created_at') {
                comparison = compareGameLogRowDates(left, right);
            } else {
                const leftType = left.type || '';
                const rightType = right.type || '';
                if (leftType !== rightType) {
                    comparison = leftType > rightType ? 1 : -1;
                }
            }
            if (comparison !== 0) {
                return desc ? -comparison : comparison;
            }
        }
        return 0;
    });
}
