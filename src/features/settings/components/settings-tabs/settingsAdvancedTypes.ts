import type { AppDataDirState } from '@/platform/tauri/bindings';
import type { AvatarAutoCleanupPreference } from '@/shared/constants/settings';

export type SettingsAdvancedPrefs = {
    anonymousUsageTelemetry?: boolean;
    autoSweepVRChatCache?: boolean;
    avatarAutoCleanup?: AvatarAutoCleanupPreference;
    gameLogDisabled?: boolean;
    feedPersistenceDisabled?: boolean;
    logResourceLoad?: boolean;
    udonExceptionLogging?: boolean;
};

export type SettingsAdvancedAction = () => void | Promise<void>;

export type SettingsAdvancedModel = {
    appDataDirState?: AppDataDirState | null;
    avatarAutoCleanupOptions: readonly AvatarAutoCleanupPreference[];
    configTreeData: Record<string, unknown>;
    onAnonymousUsageTelemetryChange: (checked: boolean) => void;
    onAutoSweepVRChatCacheChange: (checked: boolean) => void;
    onAvatarAutoCleanupChange: (value: AvatarAutoCleanupPreference) => void;
    onClearConfigTreeData: () => void;
    onCleanupAppDataDir: SettingsAdvancedAction;
    onDismissAppDataDirCleanup: SettingsAdvancedAction;
    onGameLogDisabledChange: (disabled: boolean) => void;
    onFeedPersistenceDisabledChange: (disabled: boolean) => void;
    onLogResourceLoadChange: (checked: boolean) => void;
    onMigrateLegacyVrcxData: SettingsAdvancedAction;
    onOpenAppDataDirSelector: SettingsAdvancedAction;
    onOpenPurgeDialog: () => void;
    onRefreshConfigTreeData: SettingsAdvancedAction;
    onRefreshOnlineVisits: SettingsAdvancedAction;
    onRefreshSqliteTableSizes: SettingsAdvancedAction;
    onResetAppDataDir: SettingsAdvancedAction;
    onUdonExceptionLoggingChange: (checked: boolean) => void;
    onlineVisitCount: number | null;
    prefs: SettingsAdvancedPrefs;
    sqliteTableSizeRows: ReadonlyArray<readonly [string, string]>;
    sqliteTableSizes: Record<string, unknown>;
};
