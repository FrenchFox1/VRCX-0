import type {
    SortingState,
    ColumnVisibilityState
} from '@tanstack/react-table';

import {
    getDataTableStorageKey,
    readPersistedTableState,
    safeJsonParse,
    writePersistedTableState
} from '@/components/data-table/dataTablePersistence';
import { moderationTypes } from '@/shared/constants/moderation';
import { isRecord } from '@/shared/utils/record';

import type { ModerationRow } from './moderationPageTypes';

export const MODERATION_DEFAULT_PAGE_SIZES = [10, 15, 20, 25, 50, 100];
export const MODERATION_DEFAULT_SORTING = [
    {
        id: 'created',
        desc: true
    }
];
export const MODERATION_COLUMN_IDS = [
    'spacer',
    'created',
    'type',
    'sourceDisplayName',
    'targetDisplayName',
    'action',
    'trailing'
];
const MODERATION_SORTING_COLUMN_IDS = MODERATION_COLUMN_IDS.filter(
    (columnId) =>
        columnId !== 'sourceDisplayName' && columnId !== 'targetDisplayName'
);
export const MODERATION_TYPE_FILTERS_CONFIG_KEY =
    'VRCX_playerModerationTableFilters';

const MODERATION_STORAGE_KEY = getDataTableStorageKey('moderation');
const TYPE_LABELS: Record<string, string> = {
    block: 'Block',
    unblock: 'Unblock',
    mute: 'Mute',
    unmute: 'Unmute',
    interactOn: 'Interact On',
    interactOff: 'Interact Off',
    muteChat: 'Mute Chat',
    unmuteChat: 'Unmute Chat'
};

export function readModerationPersistedState() {
    return readPersistedTableState(MODERATION_STORAGE_KEY);
}

export function writeModerationPersistedState(patch: Record<string, unknown>) {
    writePersistedTableState(MODERATION_STORAGE_KEY, patch);
}

export function resolveModerationTypeLabel(
    type: string,
    t: (key: string) => string
) {
    if (!type) {
        return '';
    }
    const key = `view.moderation.filters.${type}`;
    const label = t(key);
    return label && label !== key ? label : TYPE_LABELS[type] || type;
}

export function sanitizeModerationSorting(value: unknown): SortingState {
    if (!Array.isArray(value)) {
        return MODERATION_DEFAULT_SORTING;
    }
    const filtered = value.filter(
        (entry): entry is SortingState[number] =>
            isRecord(entry) &&
            typeof entry.id === 'string' &&
            MODERATION_SORTING_COLUMN_IDS.includes(entry.id)
    );
    return filtered.length ? filtered : MODERATION_DEFAULT_SORTING;
}

export function sanitizeModerationPageSizes(value: unknown): number[] {
    if (!Array.isArray(value)) {
        return MODERATION_DEFAULT_PAGE_SIZES;
    }
    const normalized = Array.from(
        new Set(
            value
                .map((entry) => Number.parseInt(String(entry), 10))
                .filter(
                    (entry) =>
                        Number.isFinite(entry) && entry > 0 && entry <= 1000
                )
        )
    ).sort((left, right) => left - right);
    return normalized.length ? normalized : MODERATION_DEFAULT_PAGE_SIZES;
}

export function sanitizeModerationColumnVisibility(
    value: unknown
): ColumnVisibilityState {
    const visibility: ColumnVisibilityState = {};
    if (!isRecord(value)) {
        return visibility;
    }
    for (const columnId of MODERATION_COLUMN_IDS) {
        if (typeof value[columnId] === 'boolean') {
            visibility[columnId] = value[columnId];
        }
    }
    return visibility;
}

export function sanitizeModerationColumnOrder(value: unknown): string[] {
    if (!Array.isArray(value)) {
        return MODERATION_COLUMN_IDS;
    }
    const orderedColumns = value.filter(
        (columnId): columnId is string =>
            typeof columnId === 'string' &&
            MODERATION_COLUMN_IDS.includes(columnId)
    );
    const missingColumns = MODERATION_COLUMN_IDS.filter(
        (columnId) => !orderedColumns.includes(columnId)
    );
    return [...orderedColumns, ...missingColumns];
}

export function resolveModerationPageSize(
    candidate: unknown,
    pageSizes: readonly number[],
    fallback: number = MODERATION_DEFAULT_PAGE_SIZES[1]
): number {
    const fallbackPageSize = pageSizes.length
        ? pageSizes[0]
        : MODERATION_DEFAULT_PAGE_SIZES[0];
    const nearestPageSize = (value: number) =>
        pageSizes.length
            ? pageSizes.reduce((previous, size) =>
                  Math.abs(size - value) < Math.abs(previous - value)
                      ? size
                      : previous
              )
            : fallbackPageSize;
    const parsed = Number.parseInt(String(candidate), 10);
    if (Number.isFinite(parsed) && parsed > 0) {
        return pageSizes.includes(parsed) ? parsed : nearestPageSize(parsed);
    }
    if (pageSizes.includes(fallback)) {
        return fallback;
    }
    return nearestPageSize(fallback);
}

export function normalizeModerationSelectedTypes(value: unknown): string[] {
    if (!Array.isArray(value)) {
        return [];
    }
    return value.filter(
        (entry): entry is string =>
            typeof entry === 'string' && moderationTypes.includes(entry)
    );
}

export function parseModerationSelectedTypes(value: unknown) {
    return normalizeModerationSelectedTypes(safeJsonParse(value));
}

export function matchesModerationSearch(
    row: ModerationRow,
    searchQuery: string
) {
    if (!searchQuery) {
        return true;
    }
    const query = searchQuery.trim().toLowerCase();
    if (!query) {
        return true;
    }
    return (
        row.sourceDisplayName.toLowerCase().includes(query) ||
        row.targetDisplayName.toLowerCase().includes(query)
    );
}

export function getModerationRowKey(row: ModerationRow) {
    if (row.id) {
        return `${row.id}:${row.type}`;
    }
    return [row.type, row.sourceUserId, row.targetUserId, row.created].join(
        ':'
    );
}

export function isSameModerationRow(left: ModerationRow, right: ModerationRow) {
    return getModerationRowKey(left) === getModerationRowKey(right);
}
