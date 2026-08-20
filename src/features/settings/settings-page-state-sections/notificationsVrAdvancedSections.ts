import {
    avatarAutoCleanupOptions,
    desktopToastOptions,
    notificationTtsNameModeOptions,
    notificationTtsOptions,
    sqliteTableSizeRows
} from '../settingsOptions';
import type { BuildSettingsPageStateSectionsInput } from '../settingsPageStateSections';

export function buildNotificationsSection({
    ttsVoices,
    notificationTtsTestVisible,
    notificationTtsTest,
    setDesktopNotificationsDialogOpen,
    setTtsNotificationsDialogOpen,
    saveStringPreference,
    saveBoolPreference,
    saveNotificationTtsMode,
    saveNotificationTtsVoice,
    setNotificationTtsTestVisible,
    setNotificationTtsTest,
    speakNotificationTts
}: BuildSettingsPageStateSectionsInput) {
    return {
        desktopToastOptions,
        notificationTtsOptions,
        notificationTtsNameModeOptions,
        ttsVoices,
        notificationTtsTestVisible,
        notificationTtsTest,
        setDesktopNotificationsDialogOpen,
        setTtsNotificationsDialogOpen,
        saveStringPreference,
        saveBoolPreference,
        saveNotificationTtsMode,
        saveNotificationTtsVoice,
        setNotificationTtsTestVisible,
        setNotificationTtsTest,
        speakNotificationTts
    };
}

export function buildVrSection({
    setVrNotificationsDialogOpen,
    setHmdNotificationsDialogOpen,
    setWristFeedNotificationsDialogOpen,
    savePreferenceValue,
    saveStringPreference,
    saveBoolPreference,
    setIntConfigPreference,
    saveWristOverlayEnabled
}: BuildSettingsPageStateSectionsInput) {
    return {
        setVrNotificationsDialogOpen,
        setHmdNotificationsDialogOpen,
        setWristFeedNotificationsDialogOpen,
        savePreferenceValue,
        saveStringPreference,
        saveBoolPreference,
        setIntConfigPreference,
        saveWristOverlayEnabled
    };
}

export function buildAdvancedSection({
    sqliteTableSizes,
    onlineVisitCount,
    configTreeData,
    appDataDirState,
    saveBoolPreference,
    handleGameLogDisabledChange,
    handleFeedPersistenceDisabledChange,
    handleAvatarFeedPersistenceDisabledChange,
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
}: BuildSettingsPageStateSectionsInput) {
    return {
        avatarAutoCleanupOptions,
        sqliteTableSizes,
        sqliteTableSizeRows,
        onlineVisitCount,
        configTreeData,
        appDataDirState,
        saveBoolPreference,
        handleGameLogDisabledChange,
        handleFeedPersistenceDisabledChange,
        handleAvatarFeedPersistenceDisabledChange,
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
        migrateLegacyVrcxData,
        onAnonymousUsageTelemetryChange: (checked: boolean) => {
            saveBoolPreference(
                'anonymousUsageTelemetry',
                'anonymousUsageTelemetry',
                checked
            );
        }
    };
}
