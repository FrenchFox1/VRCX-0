import type { FavoriteGroupMap } from '@/domain/favorites/types';

import {
    normalizeFriendsLocationId,
    resolveLocationSummary
} from './friendsLocationsRows';

export function buildFriendsLocationsFavoriteIdSet(
    remoteFavoriteIds: readonly string[] = [],
    localFriendFavorites: FavoriteGroupMap = {}
): Set<string> {
    const ids = new Set<string>();

    for (const id of remoteFavoriteIds ?? []) {
        const normalized = normalizeFriendsLocationId(id);
        if (normalized) {
            ids.add(normalized);
        }
    }

    for (const groupIds of Object.values(localFriendFavorites ?? {})) {
        for (const id of groupIds) {
            const normalized = normalizeFriendsLocationId(id);
            if (normalized) {
                ids.add(normalized);
            }
        }
    }

    return ids;
}

export function matchesFriendLocationSearch(
    friend: Record<string, unknown> | null | undefined,
    searchQuery: string,
    favoriteIds: ReadonlySet<string>
): boolean {
    if (!searchQuery) {
        return true;
    }

    const location = resolveLocationSummary(friend);
    const query = searchQuery.trim().toLowerCase();
    if (!query) {
        return true;
    }

    return (
        String(friend?.displayName || '')
            .toLowerCase()
            .includes(query) ||
        String(friend?.username || '')
            .toLowerCase()
            .includes(query) ||
        String(friend?.statusDescription || '')
            .toLowerCase()
            .includes(query) ||
        String(friend?.worldId || '')
            .toLowerCase()
            .includes(query) ||
        String(friend?.location || '')
            .toLowerCase()
            .includes(query) ||
        String(location.label || '')
            .toLowerCase()
            .includes(query) ||
        String(location.meta || '')
            .toLowerCase()
            .includes(query) ||
        (query === 'favorite' &&
            favoriteIds.has(normalizeFriendsLocationId(friend?.id)))
    );
}
