import { useEffect, useRef, useState } from 'react';

import { commands, type AvatarUsageRow } from '@/platform/tauri/bindings';
import avatarProfileRepository from '@/repositories/avatarProfileRepository';

import { resolveMissingEntities } from './resolveMissingEntities';

const AVATAR_LIMIT = 10;

export function useActivityAvatarUsage(
    ownerUserId: string,
    enabled: boolean
): AvatarUsageRow[] {
    const [rows, setRows] = useState<AvatarUsageRow[]>([]);
    const fetchedRef = useRef(new Set<string>());

    useEffect(() => {
        if (!ownerUserId || !enabled) {
            return;
        }
        let active = true;
        const isActive = () => active;

        void commands
            .appAvatarUsageRanking(ownerUserId, AVATAR_LIMIT)
            .then(async (ranking) => {
                if (!active) {
                    return;
                }
                setRows(ranking);

                const missing = ranking
                    .filter(
                        (row) =>
                            !row.name && !fetchedRef.current.has(row.avatarId)
                    )
                    .map((row) => row.avatarId);
                for (const id of missing) {
                    fetchedRef.current.add(id);
                }
                await resolveMissingEntities({
                    ids: missing,
                    isActive,
                    fetchOne: async (avatarId) => {
                        const profile =
                            await avatarProfileRepository.getAvatarProfile({
                                avatarId
                            });
                        return profile?.name ? profile : null;
                    },
                    onResolved: (avatarId, profile) => {
                        setRows((previous) =>
                            previous.map((row) =>
                                row.avatarId === avatarId
                                    ? {
                                          ...row,
                                          name: profile.name,
                                          thumbnailImageUrl:
                                              profile.thumbnailImageUrl ||
                                              row.thumbnailImageUrl,
                                          imageUrl:
                                              profile.imageUrl || row.imageUrl
                                      }
                                    : row
                            )
                        );
                    }
                });
            })
            .catch(() => {});

        return () => {
            active = false;
        };
    }, [enabled, ownerUserId]);

    return rows;
}
