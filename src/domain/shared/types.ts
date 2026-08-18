export const LOAD_STATUSES = ['idle', 'running', 'ready', 'error'] as const;

export type LoadStatus = (typeof LOAD_STATUSES)[number];

export function isLoadStatus(value: string): value is LoadStatus {
    return (LOAD_STATUSES as readonly string[]).includes(value);
}
