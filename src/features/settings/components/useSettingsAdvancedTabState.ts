import { useShallow } from 'zustand/react/shallow';

import type { AvatarAutoCleanupPreference } from '@/shared/constants/settings';
import { usePreferencesStore } from '@/state/preferencesStore';

import { useSettingsPageSection } from '../SettingsPageStateContext';

export function useSettingsAdvancedTabState() {
    const advanced = useSettingsPageSection('advanced');
    const prefs = usePreferencesStore(
        useShallow((state) => ({
            autoSweepVRChatCache: state.autoSweepVRChatCache,
            avatarAutoCleanup: state.avatarAutoCleanup,
            gameLogDisabled: state.gameLogDisabled,
            feedPersistenceDisabled: state.feedPersistenceDisabled,
            anonymousUsageTelemetry: state.anonymousUsageTelemetry,
            udonExceptionLogging: state.udonExceptionLogging,
            logResourceLoad: state.logResourceLoad
        }))
    );
    const {
        avatarAutoCleanupOptions,
        sqliteTableSizes,
        sqliteTableSizeRows,
        onlineVisitCount,
        configTreeData,
        appDataDirState,
        saveBoolPreference,
        handleGameLogDisabledChange,
        handleFeedPersistenceDisabledChange,
        saveStringPreference,
        setPurgeDialogOpen,
        refreshSqliteTableSizes,
        refreshOnlineVisits,
        refreshConfigTreeData,
        openAppDataDirSelector,
        resetAppDataDir,
        cleanupAppDataDir,
        dismissAppDataDirCleanup,
        setConfigTreeData,
        migrateLegacyVrcxData
    } = advanced;

    const advancedTab = {
        prefs,
        avatarAutoCleanupOptions,
        sqliteTableSizes,
        sqliteTableSizeRows,
        onlineVisitCount,
        configTreeData,
        appDataDirState,
        onAutoSweepVRChatCacheChange: (checked: boolean) => {
            saveBoolPreference(
                'autoSweepVRChatCache',
                'VRCX_autoSweepVRChatCache',
                checked
            );
        },
        onUdonExceptionLoggingChange: (checked: boolean) => {
            saveBoolPreference(
                'udonExceptionLogging',
                'VRCX_udonExceptionLogging',
                checked
            );
        },
        onLogResourceLoadChange: (checked: boolean) => {
            saveBoolPreference('logResourceLoad', 'logResourceLoad', checked);
        },
        onAnonymousUsageTelemetryChange: (checked: boolean) => {
            saveBoolPreference(
                'anonymousUsageTelemetry',
                'anonymousUsageTelemetry',
                checked
            );
        },
        onGameLogDisabledChange: (checked: boolean) => {
            handleGameLogDisabledChange(checked);
        },
        onFeedPersistenceDisabledChange: (checked: boolean) => {
            handleFeedPersistenceDisabledChange(checked);
        },
        onAvatarAutoCleanupChange: (value: AvatarAutoCleanupPreference) => {
            saveStringPreference(
                'avatarAutoCleanup',
                'avatarAutoCleanup',
                value
            );
        },
        onOpenPurgeDialog: () => setPurgeDialogOpen(true),
        onMigrateLegacyVrcxData: migrateLegacyVrcxData,
        onRefreshSqliteTableSizes: refreshSqliteTableSizes,
        onRefreshOnlineVisits: refreshOnlineVisits,
        onRefreshConfigTreeData: refreshConfigTreeData,
        onOpenAppDataDirSelector: openAppDataDirSelector,
        onResetAppDataDir: resetAppDataDir,
        onCleanupAppDataDir: cleanupAppDataDir,
        onDismissAppDataDirCleanup: dismissAppDataDirCleanup,
        onClearConfigTreeData: () => setConfigTreeData({})
    };

    return advancedTab;
}
