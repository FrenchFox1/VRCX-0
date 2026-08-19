import { commands } from '@/platform/tauri/bindings';
import type { MutualGraphFetchStatus } from '@/platform/tauri/bindings';
import { useRuntimeStore } from '@/state/runtimeStore';

type StartMutualGraphFetchInput = {
    ownerUserId: string;
    endpoint?: string;
    friendIds: string[];
};

const TERMINAL_STATUSES = new Set(['completed', 'cancelled', 'error']);
const ACTIVE_STATUSES = new Set(['running', 'cancelling']);
const TERMINAL_RESET_DELAY_MS = 5000;

let resetTimer: number | null = null;
let latestAcceptedRunId = 0;
let latestAcceptedRevision = 0;
const sessionStartedRunIds = new Set<number>();

function normalizeStatus(status: MutualGraphFetchStatus) {
    return {
        ...status,
        startedAt: status.startedAt || null,
        updatedAt: status.updatedAt || null
    };
}

function isNewerStatus(runId: number, revision: number): boolean {
    return (
        runId > latestAcceptedRunId ||
        (runId === latestAcceptedRunId && revision > latestAcceptedRevision)
    );
}

function clearResetTimer() {
    if (resetTimer !== null) {
        window.clearTimeout(resetTimer);
        resetTimer = null;
    }
}

function scheduleTerminalReset() {
    clearResetTimer();
    resetTimer = window.setTimeout(() => {
        resetTimer = null;
        useRuntimeStore.getState().resetMutualGraphState();
    }, TERMINAL_RESET_DELAY_MS);
}

function applyMutualGraphFetchStatus(
    status: MutualGraphFetchStatus
) {
    const normalized = normalizeStatus(status);
    if (!isNewerStatus(normalized.runId, normalized.revision)) {
        return normalized;
    }
    latestAcceptedRunId = normalized.runId;
    latestAcceptedRevision = normalized.revision;
    useRuntimeStore.getState().setMutualGraphState(normalized);
    if (ACTIVE_STATUSES.has(normalized.status)) {
        clearResetTimer();
    } else if (TERMINAL_STATUSES.has(normalized.status)) {
        scheduleTerminalReset();
    }
    return normalized;
}

export function handleMutualGraphFetchStatusEvent(
    status: MutualGraphFetchStatus
) {
    return applyMutualGraphFetchStatus(status);
}

export async function refreshMutualGraphFetchStatus() {
    const status = await commands.appMutualGraphFetchStatusGet();
    return applyMutualGraphFetchStatus(status);
}

export async function startMutualGraphFetch({
    ownerUserId,
    endpoint = '',
    friendIds
}: StartMutualGraphFetchInput) {
    const status = await commands.appMutualGraphFetchStart({
        ownerUserId,
        endpoint,
        friendIds
    });
    const runId = status.runId;
    if (runId) {
        sessionStartedRunIds.add(runId);
    }
    return applyMutualGraphFetchStatus(status);
}

export async function cancelMutualGraphFetch(ownerUserId: string) {
    const status = await commands.appMutualGraphFetchCancel({
        ownerUserId
    });
    return applyMutualGraphFetchStatus(status);
}

export function wasMutualGraphFetchStartedInThisSession(runId: number) {
    return sessionStartedRunIds.has(runId);
}
