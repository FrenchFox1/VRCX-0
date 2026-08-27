import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

import {
    UPDATE_AVAILABLE_TOAST_ID,
    openOrInstallLatestAvailableUpdate,
    shouldShowUpdateUi
} from '@/services/updateInstallService';
import { useRuntimeStore } from '@/state/runtimeStore';

type UpdateLoopState = ReturnType<
    typeof useRuntimeStore.getState
>['updateLoop'];
type UpdateLoopRelease = NonNullable<UpdateLoopState['latestUpdaterRelease']>;

function getLatestUpdaterDisplayVersion(release: UpdateLoopRelease) {
    return (
        (
            release.latestVersion ||
            release.displayVersion ||
            release.canonicalVersion ||
            release.tagName
        ).trim() || '-'
    );
}

function formatUpdateVersion(version: string) {
    if (!version || version === '-') {
        return version;
    }
    return version.replace(/^v/i, '');
}

function isDownloadedUpdateReady({
    latestUpdaterRelease,
    autoDownloadState,
    downloadedVersion
}: {
    latestUpdaterRelease: UpdateLoopRelease;
    autoDownloadState: UpdateLoopState['autoDownloadState'];
    downloadedVersion: UpdateLoopState['downloadedVersion'];
}) {
    const latestVersion = latestUpdaterRelease.canonicalVersion;
    return (
        autoDownloadState === 'downloaded' &&
        Boolean(latestVersion) &&
        downloadedVersion === latestVersion
    );
}

export function showUpdateAvailableToast({
    latestUpdaterRelease,
    t,
    onUpdate
}: {
    latestUpdaterRelease: UpdateLoopRelease;
    t: (key: string, values?: Record<string, unknown>) => string;
    onUpdate: () => void;
}) {
    toast.info(
        t('service.background_maintenance.label.vrcx_update_available'),
        {
            id: UPDATE_AVAILABLE_TOAST_ID,
            icon: null,
            description: formatUpdateVersion(
                getLatestUpdaterDisplayVersion(latestUpdaterRelease)
            ),
            duration: Infinity,
            position: 'bottom-right',
            closeButton: true,
            dismissible: true,
            action: {
                label: t('nav_menu.update'),
                onClick: onUpdate
            }
        }
    );
}

export function showUpdateReadyToast({
    latestUpdaterRelease,
    t,
    onUpdate
}: {
    latestUpdaterRelease: UpdateLoopRelease;
    t: (key: string, values?: Record<string, unknown>) => string;
    onUpdate: () => void;
}) {
    const version = formatUpdateVersion(
        getLatestUpdaterDisplayVersion(latestUpdaterRelease)
    );
    toast.success(
        t('dialog.vrcx_updater.ready_for_update', {
            value: version
        }),
        {
            id: UPDATE_AVAILABLE_TOAST_ID,
            description: undefined,
            duration: Infinity,
            position: 'bottom-right',
            closeButton: true,
            dismissible: true,
            action: {
                label: t('nav_menu.update_downloaded'),
                onClick: onUpdate
            }
        }
    );
}

export function UpdateAvailableToastHost(): null {
    const { t } = useTranslation();
    const showUpdateUi = useRuntimeStore((state) =>
        shouldShowUpdateUi(state.updateLoop)
    );
    const latestUpdaterRelease = useRuntimeStore(
        (state) => state.updateLoop.latestUpdaterRelease
    );
    const autoDownloadState = useRuntimeStore(
        (state) => state.updateLoop.autoDownloadState
    );
    const downloadedVersion = useRuntimeStore(
        (state) => state.updateLoop.downloadedVersion
    );

    useEffect(() => {
        if (!showUpdateUi || !latestUpdaterRelease) {
            toast.dismiss(UPDATE_AVAILABLE_TOAST_ID);
            return undefined;
        }

        const openLatestUpdate = () => {
            void openOrInstallLatestAvailableUpdate({
                toastId: UPDATE_AVAILABLE_TOAST_ID
            });
        };
        const showToast = isDownloadedUpdateReady({
            latestUpdaterRelease,
            autoDownloadState,
            downloadedVersion
        })
            ? showUpdateReadyToast
            : showUpdateAvailableToast;
        showToast({
            latestUpdaterRelease,
            t,
            onUpdate: openLatestUpdate
        });

        return undefined;
    }, [
        autoDownloadState,
        downloadedVersion,
        showUpdateUi,
        latestUpdaterRelease,
        t
    ]);

    return null;
}
