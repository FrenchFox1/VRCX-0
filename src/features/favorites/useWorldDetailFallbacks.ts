import { useMemo } from 'react';

import type { FavoriteKind } from '@/domain/favorites/types';
import type { LoadStatus } from '@/domain/shared/types';
import worldProfileRepository from '@/repositories/worldProfileRepository';

import {
    type DetailMap,
    getRemoteEntityCacheFallbackIds,
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
