import { useMemo } from 'react';

import type { FavoriteKind } from '@/domain/favorites/types';
import type { LoadStatus } from '@/domain/shared/types';
import worldProfileRepository from '@/repositories/worldProfileRepository';

import {
    type DetailMap,
    filterRemoteEntityCacheFallbacksById,
    getRemoteEntityCacheFallbackIds,
    loadRemoteEntityCacheFallbacksById,
    useRemoteEntityCacheFallbackLoader
} from './remoteEntityCacheFallbacks';

type WorldDetailFallbackInput = {
    worldIds: string[];
    kind: FavoriteKind;
    remoteEntityDetailsData?: DetailMap;
    remoteEntityDetailsStatus: LoadStatus;
};

const fetchWorldById = (worldId: string) =>
    worldProfileRepository.getWorldProfile({ worldId });

export function getWorldDetailFallbackIds({
    worldIds,
    kind,
    remoteEntityDetailsData,
    remoteEntityDetailsStatus
}: WorldDetailFallbackInput): string[] {
    return getRemoteEntityCacheFallbackIds({
        entityIds: worldIds,
        detailSources: [remoteEntityDetailsData],
        isReady: kind === 'world' && remoteEntityDetailsStatus === 'ready'
    });
}

export const filterWorldDetailFallbacksById =
    filterRemoteEntityCacheFallbacksById;

export function loadWorldDetailFallbacksById(
    worldIds: string[]
): Promise<DetailMap> {
    return loadRemoteEntityCacheFallbacksById(worldIds, fetchWorldById);
}

export function useWorldDetailFallbacks({
    worldIds,
    kind,
    remoteEntityDetailsData,
    remoteEntityDetailsStatus
}: WorldDetailFallbackInput): DetailMap {
    const fallbackWorldIds = useMemo(
        () =>
            getWorldDetailFallbackIds({
                worldIds,
                kind,
                remoteEntityDetailsData,
                remoteEntityDetailsStatus
            }),
        [worldIds, kind, remoteEntityDetailsData, remoteEntityDetailsStatus]
    );

    return useRemoteEntityCacheFallbackLoader(fallbackWorldIds, fetchWorldById);
}
