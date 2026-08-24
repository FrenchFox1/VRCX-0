import { useEffect, useState } from 'react';

import { commands } from '@/platform/tauri/bindings';

export type ActivityWorldSummary = {
    name: string;
    thumbnailUrl: string;
    imageUrl: string;
    authorName: string;
    description: string;
};

export function useActivityWorldNames(
    worldIds: string[]
): Map<string, ActivityWorldSummary> {
    const [summaries, setSummaries] = useState<
        Map<string, ActivityWorldSummary>
    >(new Map());
    const worldIdsKey = worldIds.join(',');

    useEffect(() => {
        const ids = worldIdsKey ? worldIdsKey.split(',').filter(Boolean) : [];
        if (ids.length === 0) {
            return;
        }
        let active = true;
        void commands
            .appWorldSummariesGet(ids)
            .then((rows) => {
                if (!active) {
                    return;
                }
                setSummaries((previous) => {
                    const next = new Map(previous);
                    for (const id of ids) {
                        const row = rows[id];
                        if (row) {
                            next.set(id, {
                                name: row.name,
                                thumbnailUrl:
                                    row.thumbnailImageUrl || row.imageUrl || '',
                                imageUrl:
                                    row.imageUrl || row.thumbnailImageUrl || '',
                                authorName: row.authorName,
                                description: row.description
                            });
                        }
                    }
                    return next;
                });
            })
            .catch(() => {});
        return () => {
            active = false;
        };
    }, [worldIdsKey]);

    return summaries;
}
