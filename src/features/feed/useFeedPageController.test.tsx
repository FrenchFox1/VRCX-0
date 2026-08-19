// @vitest-environment jsdom

import type {
    ColumnOrderState,
    ColumnSizingState,
    ColumnVisibilityState,
    ExpandedState,
    PaginationState,
    SortingState
} from '@tanstack/react-table';
import { act, renderHook } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import type { FeedRow } from './feedTypes';

const mocks = vi.hoisted(() => ({
    rows: [] as FeedRow[]
}));

vi.mock('./components/FeedColumns', () => ({
    useFeedColumns: () => [
        {
            accessorKey: 'type',
            id: 'type'
        }
    ]
}));

vi.mock('./useFeedFilters', () => ({
    useFeedFilters: () => ({
        activeFilters: [],
        dateFrom: '',
        dateTo: '',
        deferredSearchQuery: '',
        deferredScopedUserIds: [],
        favoritesOnly: false,
        setFavoritesOnly: vi.fn(),
        setFeedFilters: vi.fn()
    })
}));

vi.mock('./useFeedFriendActions', () => ({
    useFeedFriendActions: () => ({})
}));

vi.mock('./useFeedPreviousInstancesDialog', () => ({
    useFeedPreviousInstancesDialog: () => ({
        loadingKey: '',
        openPreviousInstancesForLocation: vi.fn()
    })
}));

vi.mock('./useFeedRows', () => ({
    useFeedRows: () => ({
        friendLogNamesById: {},
        isFavoritesLoaded: true,
        loadStatus: 'ready',
        rows: mocks.rows
    })
}));

vi.mock('./useFeedTableMeta', () => ({
    useFeedTableMeta: () => ({})
}));

vi.mock('./useFeedTableState', () => ({
    useFeedTableState: () => {
        const [columnOrder, setColumnOrder] = useState<ColumnOrderState>([]);
        const [columnOrderLocked, setColumnOrderLocked] = useState(false);
        const [columnSizing, setColumnSizing] = useState<ColumnSizingState>({});
        const [columnVisibility, setColumnVisibility] =
            useState<ColumnVisibilityState>({});
        const [expanded, setExpanded] = useState<ExpandedState>({});
        const [pagination, setPagination] = useState<PaginationState>({
            pageIndex: 0,
            pageSize: 20
        });
        const [sorting, setSorting] = useState<SortingState>([]);

        return {
            columnOrder,
            columnOrderLocked,
            columnSizing,
            columnVisibility,
            expanded,
            pageSizes: [20],
            pagination,
            preferencesReady: true,
            setColumnOrder,
            setColumnOrderLocked,
            setColumnSizing,
            setColumnVisibility,
            setExpanded,
            setPagination,
            setSorting,
            sorting
        };
    }
}));

import { getFeedRowId } from './feedRows';
import { useFeedPageController } from './useFeedPageController';

describe('useFeedPageController', () => {
    it('keeps a row expanded when refreshed data retains its id', async () => {
        const row: FeedRow = {
            rowId: 1,
            sourceRank: 60,
            type: 'GPS',
            previousLocation: 'private'
        };
        const rowId = getFeedRowId(row);
        mocks.rows = [row];
        const { result, rerender } = renderHook(() =>
            useFeedPageController({ routeScopedUserIds: [] })
        );

        act(() => {
            result.current.table.getRow(rowId).toggleExpanded(true);
        });
        expect(result.current.table.getRow(rowId).getIsExpanded()).toBe(true);

        mocks.rows = [{ ...row, worldName: 'Refreshed World' }];
        rerender();
        act(() => {
            result.current.table.getRowModel();
        });
        await act(async () => {
            await Promise.resolve();
        });

        expect(result.current.table.getRow(rowId).getIsExpanded()).toBe(true);
    });
});
