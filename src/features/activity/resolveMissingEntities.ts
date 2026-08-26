export async function resolveMissingEntities<T>({
    ids,
    isActive,
    fetchOne,
    onResolved
}: {
    ids: string[];
    isActive: () => boolean;
    fetchOne: (id: string) => Promise<T | null>;
    onResolved: (id: string, value: T) => void;
}): Promise<void> {
    const resolved = await Promise.all(
        ids.map((id) =>
            fetchOne(id)
                .then((value) => [id, value] as const)
                .catch(() => [id, null] as const)
        )
    );
    if (!isActive()) {
        return;
    }
    for (const [id, value] of resolved) {
        if (value) {
            onResolved(id, value);
        }
    }
}
