import { describe, expect, it, vi } from 'vitest';

import type { AppToastOptions } from '@/services/toastService';

import type { GalleryInventoryActionDeps } from './galleryTypes';
import { useGalleryInventoryActions } from './useGalleryInventoryActions';

function useActions(overrides: Partial<GalleryInventoryActionDeps> = {}) {
    const nextUser = {
        id: 'usr_self',
        displayName: 'Current User',
        userIcon: 'https://api.vrchat.cloud/api/1/file/file_icon/1'
    };
    const updateCurrentUser = vi.fn().mockResolvedValue(nextUser);
    const setAuthBootstrap = vi.fn();
    const refreshMediaProfile = vi
        .fn()
        .mockResolvedValue({ userIcon: 'canonical' });
    const toast = {
        error: vi.fn(),
        success: vi.fn(),
        add(options: AppToastOptions) {
            if (options.type === 'error') {
                toast.error(options);
                return '';
            }
            if (options.type === 'success') {
                toast.success(options);
                return '';
            }
            throw new Error('Unhandled toast type: ' + options.type);
        }
    };
    const currentUserSnapshot = {
        id: 'usr_self',
        $isVRCPlus: false,
        tags: []
    };
    const actions = useGalleryInventoryActions({
        buildProfilePicOverride: (endpoint: string, fileId: string) =>
            fileId ? `${endpoint}/file/${fileId}/1` : '',
        currentUserProfileService: {
            updateCurrentUser
        },
        currentEndpoint: 'https://api.vrchat.cloud/api/1',
        currentUserId: 'usr_self',
        mediaProfile: {},
        refreshMediaProfile,
        confirm: vi.fn(),
        getAuthTarget: () => ({
            userId: 'usr_self',
            endpoint: 'https://api.vrchat.cloud/api/1'
        }),
        isRuntimeAuthTarget: () => true,
        mediaRepository: {
            consumeInventoryBundle: vi.fn(),
            deletePrint: vi.fn(),
            redeemReward: vi.fn(),
            setPrintFavorite: vi.fn()
        },
        prompt: vi.fn(),
        refreshInventory: vi.fn(),
        setAssets: vi.fn(),
        setMutatingKey: vi.fn(),
        t: (key: string) => key,
        toast,
        useRuntimeStore: {
            getState: () => ({
                auth: {
                    currentUserSnapshot
                },
                setAuthBootstrap
            })
        },
        ...overrides
    });

    return {
        actions,
        nextUser,
        updateCurrentUser,
        setAuthBootstrap,
        toast,
        refreshMediaProfile
    };
}

describe('useGalleryInventoryActions', () => {
    it('allows a non-VRC+ user to set profile icons and banners', async () => {
        const {
            actions,
            nextUser,
            updateCurrentUser,
            setAuthBootstrap,
            toast,
            refreshMediaProfile
        } = useActions();

        await actions.setProfileField('userIcon', 'file_icon');

        expect(updateCurrentUser).toHaveBeenCalledWith({
            userId: 'usr_self',
            params: {
                userIcon: 'https://api.vrchat.cloud/api/1/file/file_icon/1'
            }
        });
        expect(setAuthBootstrap).toHaveBeenCalledWith({
            currentUserSnapshot: expect.objectContaining({
                ...nextUser,
                userIcon: 'canonical',
                bannerCustomUrl: ''
            }),
            currentUserDisplayName: 'Current User'
        });
        expect(toast.error).not.toHaveBeenCalled();
        expect(refreshMediaProfile).toHaveBeenCalledOnce();
        expect(updateCurrentUser.mock.invocationCallOrder[0]).toBeLessThan(
            refreshMediaProfile.mock.invocationCallOrder[0]
        );
        expect(refreshMediaProfile.mock.invocationCallOrder[0]).toBeLessThan(
            setAuthBootstrap.mock.invocationCallOrder[0]
        );
        expect(toast.success).toHaveBeenCalledWith(
            expect.objectContaining({
                type: 'success',
                title: 'message.gallery.profile_icon_changed'
            })
        );

        await actions.setProfileField('profilePicOverride', 'file_banner');

        expect(updateCurrentUser).toHaveBeenLastCalledWith({
            userId: 'usr_self',
            params: {
                profilePicOverride:
                    'https://api.vrchat.cloud/api/1/file/file_banner/1'
            }
        });
        expect(toast.success).toHaveBeenLastCalledWith(
            expect.objectContaining({
                type: 'success',
                title: 'message.gallery.profile_pic_changed'
            })
        );
    });
    it('compares the selected banner file and can clear it using the existing write field', async () => {
        const { actions, updateCurrentUser } = useActions({
            mediaProfile: { bannerCustomUrl: 'https://image/file_banner/3/256' }
        });
        await actions.setProfileField('profilePicOverride', 'file_banner');
        expect(updateCurrentUser).not.toHaveBeenCalled();
        await actions.setProfileField('profilePicOverride', '');
        expect(updateCurrentUser).toHaveBeenCalledWith({
            userId: 'usr_self',
            params: { profilePicOverride: '' }
        });
    });

    it('keeps the current snapshot and reports a failed profile reread', async () => {
        const { actions, setAuthBootstrap, toast } = useActions({
            refreshMediaProfile: vi
                .fn()
                .mockRejectedValue(new Error('refresh failed'))
        });
        await actions.setProfileField('userIcon', 'file_new');
        expect(setAuthBootstrap).not.toHaveBeenCalled();
        expect(toast.success).not.toHaveBeenCalled();
        expect(toast.error).toHaveBeenCalledWith(
            expect.objectContaining({ title: 'refresh failed' })
        );
    });

    it('ignores the reread completion after switching accounts', async () => {
        let current = true;
        const { actions, setAuthBootstrap, toast } = useActions({
            isRuntimeAuthTarget: () => current,
            refreshMediaProfile: async () => {
                current = false;
                return { id: 'usr_self' };
            }
        });
        await actions.setProfileField('userIcon', 'file_new');
        expect(setAuthBootstrap).not.toHaveBeenCalled();
        expect(toast.success).not.toHaveBeenCalled();
    });
});
