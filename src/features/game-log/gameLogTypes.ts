import type { PaginationState } from '@tanstack/react-table';
import type { Dispatch, SetStateAction } from 'react';

import type { AppColumnDef } from '@/components/data-table/appTable';
import type { LoadStatus } from '@/domain/shared/types';
import type {
    GameLogSessionDto as GeneratedGameLogSession,
    GameLogSessionEventDto as GeneratedGameLogSessionEvent,
    GameLogSessionMemberDto as GeneratedGameLogSessionMember
} from '@/platform/tauri/bindings';
import type {
    GameLogFilterType as RepositoryGameLogFilterType,
    GameLogPreviousInstanceWorldRow
} from '@/repositories/gameLogRepository';

export const GAME_LOG_SESSION_FILTER_TYPES = [
    'OnPlayerJoined',
    'OnPlayerLeft',
    'VideoPlay'
] as const;

export const GAME_LOG_LIVE_REFRESH_THROTTLE_MS = 1000;

export type GameLogViewMode = 'sessions' | 'table';

export type GameLogLoadStatus = LoadStatus;

export type GameLogRow = {
    id?: unknown;
    rowId?: unknown;
    type?: unknown;
    created_at?: unknown;
    createdAt?: unknown;
    displayName?: unknown;
    userId?: unknown;
    location?: unknown;
    instanceId?: unknown;
    worldId?: unknown;
    worldName?: unknown;
    groupName?: unknown;
    videoUrl?: unknown;
    data?: unknown;
    message?: unknown;
    resourceUrl?: unknown;
    isFavorite?: boolean | null;
    isFriend?: boolean;
    [key: string]: unknown;
};

export type GameLogSessionMember = GeneratedGameLogSessionMember & {
    isFriend?: boolean;
};

export type GameLogSessionEvent = Omit<
    GeneratedGameLogSessionEvent,
    'members'
> & {
    isFriend?: boolean;
    members?: GameLogSessionMember[] | null;
};

export type GameLogSession = Omit<GeneratedGameLogSession, 'events'> & {
    events: GameLogSessionEvent[];
};

export type GameLogDetailValue = {
    primary?: unknown;
    secondary?: unknown;
};

export type GameLogPreviousInstanceRow = GameLogPreviousInstanceWorldRow;

export type GameLogColumns = AppColumnDef<GameLogRow>[];

export type GameLogPaginationSetter = Dispatch<SetStateAction<PaginationState>>;

export type GameLogFilterType = RepositoryGameLogFilterType;

export type { PaginationState };
