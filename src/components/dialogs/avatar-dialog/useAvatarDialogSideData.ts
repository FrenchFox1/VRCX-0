import { useEffect, useState } from 'react';

import type { AvatarProfileRecord } from '@/domain/entities/avatar';
import type { LoadStatus } from '@/domain/shared/types';
import { getFileAnalysisForUnityPackages } from '@/lib/fileAnalysis';
import avatarProfileRepository from '@/repositories/avatarProfileRepository';

import { avatarGalleryImageUrl, defaultAvatarSideData } from './avatarAssets';
import { readAvatarCacheInfo } from './avatarCacheAdapter';

type AvatarSideDataProfile = Pick<
    AvatarProfileRecord,
    'id' | 'unityPackages' | 'updated_at' | 'version'
> & {
    assetUrl?: unknown;
};

export function useAvatarDialogSideData({
    avatar,
    currentEndpoint,
    galleryActive,
    sdkUnityVersion
}: {
    avatar: AvatarSideDataProfile | null;
    currentEndpoint: string;
    galleryActive: boolean;
    sdkUnityVersion: string;
}) {
    const [avatarSideData, setAvatarSideData] = useState(() =>
        defaultAvatarSideData()
    );
    const [galleryStatus, setGalleryStatus] = useState<LoadStatus>('idle');
    const avatarId = avatar?.id;
    const avatarAssetUrl = avatar?.assetUrl;
    const avatarUnityPackages = avatar?.unityPackages;

    useEffect(() => {
        setAvatarSideData(defaultAvatarSideData());
        setGalleryStatus('idle');
    }, [avatarId, currentEndpoint]);

    useEffect(() => {
        let active = true;

        if (!avatarId) {
            return () => {
                active = false;
            };
        }

        setAvatarSideData((current) => ({
            ...current,
            fileAnalysis: {},
            cache: defaultAvatarSideData().cache
        }));

        void readAvatarCacheInfo(
            {
                id: avatarId,
                assetUrl: avatarAssetUrl,
                unityPackages: avatarUnityPackages
            },
            sdkUnityVersion
        )
            .then((cache) => {
                if (active) {
                    setAvatarSideData((current) => ({ ...current, cache }));
                }
            })
            .catch(() => {
                if (active) {
                    setAvatarSideData((current) => ({
                        ...current,
                        cache: defaultAvatarSideData().cache
                    }));
                }
            });
        void getFileAnalysisForUnityPackages({
            unityPackages: avatarUnityPackages,
            sdkUnityVersion,
            endpoint: currentEndpoint
        })
            .then((fileAnalysis) => {
                if (active) {
                    setAvatarSideData((current) => ({
                        ...current,
                        fileAnalysis
                    }));
                }
            })
            .catch(() => {
                if (active) {
                    setAvatarSideData((current) => ({
                        ...current,
                        fileAnalysis: {}
                    }));
                }
            });

        return () => {
            active = false;
        };
    }, [
        avatar?.updated_at,
        avatar?.version,
        avatarAssetUrl,
        avatarId,
        avatarUnityPackages,
        currentEndpoint,
        sdkUnityVersion
    ]);

    useEffect(() => {
        let active = true;

        if (!avatarId || !galleryActive) {
            return () => {
                active = false;
            };
        }

        setGalleryStatus('running');
        void avatarProfileRepository
            .getAvatarGallery({ avatarId })
            .then((galleryRows) => {
                if (!active) {
                    return;
                }
                setAvatarSideData((current) => ({
                    ...current,
                    galleryRows,
                    galleryImages: galleryRows
                        .map(avatarGalleryImageUrl)
                        .filter((url): url is string => Boolean(url))
                }));
                setGalleryStatus('ready');
            })
            .catch(() => {
                if (!active) {
                    return;
                }
                setAvatarSideData((current) => ({
                    ...current,
                    galleryRows: [],
                    galleryImages: []
                }));
                setGalleryStatus('error');
            });

        return () => {
            active = false;
        };
    }, [
        avatar?.updated_at,
        avatar?.version,
        avatarId,
        currentEndpoint,
        galleryActive
    ]);

    return {
        avatarSideData,
        galleryStatus,
        setAvatarSideData
    };
}
