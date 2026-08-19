import type { ComponentPropsWithoutRef } from 'react';

import type { DataDirMigrationStatus } from '@/services/dataDirMigrationService';
import type { ProfileBackupStatus } from '@/services/profileBackupService';
import type {
    FriendProfileLoadState,
    InstanceQueueState,
    MutualGraphState,
    NowPlayingState,
    VrcStatusState
} from '@/state/runtimeStore';

export type StatusBarVisibilityKey =
    | 'vrchat'
    | 'steamvr'
    | 'proxy'
    | 'ws'
    | 'instanceQueue'
    | 'mutualGraph'
    | 'nowPlaying'
    | 'uptime'
    | 'zoom'
    | 'clocks'
    | 'servers';

export type StatusBarVisibility = Record<StatusBarVisibilityKey, boolean>;

export type StatusBarClock = {
    offset: number;
};

export type StatusBarTimezoneOption = {
    value: number;
    label: string;
};

export type StatusBarInstanceQueue = InstanceQueueState;

export type StatusBarMutualGraph = Pick<
    MutualGraphState,
    | 'cancelRequested'
    | 'failedFriends'
    | 'lastError'
    | 'processedFriends'
    | 'runId'
    | 'status'
    | 'totalFriends'
>;

export type StatusBarFriendProfileLoad = Pick<
    FriendProfileLoadState,
    'processedFriends' | 'status' | 'totalFriends'
>;

export type StatusBarWorldCollectionImport = {
    active: boolean;
    progress: number;
    total: number;
};

export type StatusBarProfileBackup = {
    status: ProfileBackupStatus;
    onOpenDetails: () => unknown;
};

export type StatusBarDataDirMigration = {
    status: DataDirMigrationStatus;
};

export type StatusBarNowPlaying = Pick<
    NowPlayingState,
    'length' | 'name' | 'position' | 'startedAt' | 'url'
>;

export type StatusBarRuntimeGameState = {
    currentLocationStartedAt?: string | null;
    currentWorldId?: string | null;
    currentWorldName?: string | null;
    lastGameLogAt?: string | null;
    lastGameLogType?: string | null;
    lastGameStartedAt?: string | null;
};

export type StatusBarRuntimeTransport = {
    websocketConnected?: boolean | null;
};

export type StatusBarProxyEditorState = {
    enabled: boolean;
    open: boolean;
    saving: boolean;
    server: string;
    testing: boolean;
};

export type StatusBarFooterModel = {
    appStartedAt: number;
    clockPopoverOpen: boolean[];
    currentLocationStartedTimestamp: number;
    currentWorld: string;
    dataDirMigration: StatusBarDataDirMigration;
    formatAppUptime: (ms: number) => string;
    formatClock: (nowMs: number, offset: number) => string;
    formatDuration: (ms: number) => string;
    formatStatusDate: (value: string | null | undefined) => string;
    gameStartedAt: number;
    instanceQueue: StatusBarInstanceQueue;
    isGameRunning: boolean | null;
    isSteamVRRunning: boolean | null;
    friendProfileLoad: StatusBarFriendProfileLoad;
    mutualGraph: StatusBarMutualGraph;
    nowPlaying: StatusBarNowPlaying;
    onOpenMediaLink: () => void;
    onOpenStatusPage: () => Promise<void>;
    onProxyDraftEnabledChange: (enabled: boolean) => void;
    onProxyDraftServerChange: (server: string) => void;
    onProxyEditorOpenChange: (open: boolean) => void;
    onProxySave: () => void;
    onProxySaveAndRestart: () => void;
    onProxyTest: () => Promise<void>;
    onSetClockPopoverValue: (index: number, open: boolean) => void;
    onSetZoomLevel: (nextZoom: number) => void;
    onStartBackgroundMode: () => void;
    onStepZoomLevel: (delta: number) => void;
    onUpdateClockTimezone: (index: number, offsetValue: string | null) => void;
    proxyEditor: StatusBarProxyEditorState;
    profileBackup: StatusBarProfileBackup;
    proxyEnabled: boolean;
    proxyServer: string;
    runtimeGameState: StatusBarRuntimeGameState;
    runtimeTransport: StatusBarRuntimeTransport;
    timezoneOptions: StatusBarTimezoneOption[];
    visibility: StatusBarVisibility;
    visibleClocks: StatusBarClock[];
    worldCollectionImport: StatusBarWorldCollectionImport;
    vrcStatus: Pick<
        VrcStatusState,
        'summary' | 'status' | 'refreshing' | 'error' | 'lastFetchedAt'
    > & {
        indicator: string;
    };
    zoomLabel: string;
    zoomLevel: number;
};

export type StatusBarFooterProps = ComponentPropsWithoutRef<'footer'> & {
    footer: StatusBarFooterModel;
};

export type StatusFormatterProps = {
    formatter: (ms: number) => string;
};

export type DurationValueProps = StatusFormatterProps & {
    active: boolean | null;
    startAtMs: number;
};
