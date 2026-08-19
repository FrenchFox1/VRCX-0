import { useEffect, useState } from 'react';

import type { LogLocationSnapshot } from '@/platform/tauri/bindings';
import { getCurrentLogLocation } from '@/services/gameLogWatcherService';

import { isLiveLocation } from './playerListRows';

function normalizeLogLocationSnapshot(
    snapshot: LogLocationSnapshot | null
): LogLocationSnapshot | null {
    if (!snapshot || !isLiveLocation(snapshot.location)) {
        return null;
    }

    return {
        ...snapshot,
        createdAt: snapshot.createdAt || new Date().toISOString()
    };
}

export function usePlayerListLogLocation({
    addGameLogEventCount,
    currentUserId,
    currentUserLocation,
    isGameRunning
}: {
    addGameLogEventCount?: unknown;
    currentUserId?: unknown;
    currentUserLocation?: unknown;
    isGameRunning: boolean;
}) {
    const [logLocationSnapshot, setLogLocationSnapshot] =
        useState<ReturnType<typeof normalizeLogLocationSnapshot>>(null);

    useEffect(() => {
        let active = true;

        if (currentUserLocation || !isGameRunning) {
            setLogLocationSnapshot(null);
            return () => {
                active = false;
            };
        }

        if (logLocationSnapshot) {
            return () => {
                active = false;
            };
        }

        getCurrentLogLocation()
            .then((snapshot) => {
                if (!active) {
                    return;
                }

                const normalized = normalizeLogLocationSnapshot(snapshot);
                const normalizedKey = JSON.stringify(normalized || null);
                setLogLocationSnapshot((previous) =>
                    JSON.stringify(previous || null) === normalizedKey
                        ? previous
                        : normalized
                );
            })
            .catch(() => {
                if (!active) {
                    return;
                }

                setLogLocationSnapshot(null);
            });

        return () => {
            active = false;
        };
    }, [
        addGameLogEventCount,
        currentUserId,
        currentUserLocation,
        isGameRunning,
        logLocationSnapshot
    ]);

    return logLocationSnapshot;
}
