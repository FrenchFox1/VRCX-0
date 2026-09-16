// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { AppToastOptions } from '@/services/toastService';

import type { UserDialogProfileRecord } from './useUserDialogProfileResource';

const mocks = vi.hoisted(() => ({
    addTags: vi.fn(),
    removeTags: vi.fn(),
    setAuthBootstrap: vi.fn(),
    toastError: vi.fn(),
    toastSuccess: vi.fn(),
    updateBadge: vi.fn(),
    getUserAppearanceProfile: vi.fn(),
    currentUserId: 'usr_self',
    updateCurrentUser: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('@/services/toastService', () => ({
    toast: {
        add: (options: AppToastOptions) => {
            switch (options.type) {
                case 'error':
                    return mocks.toastError(options);
                case 'success':
                    return mocks.toastSuccess(options);
                default:
                    throw new Error('Unhandled toast type: ' + options.type);
            }
        }
    }
}));

vi.mock('@/services/currentUserProfileService', () => ({
    default: {
        addCurrentUserTags: mocks.addTags,
        removeCurrentUserTags: mocks.removeTags,
        updateCurrentUser: mocks.updateCurrentUser
    }
}));

vi.mock('@/repositories/userProfileRepository', () => ({
    default: {
        updateCurrentUserBadge: mocks.updateBadge,
        getUserAppearanceProfile: mocks.getUserAppearanceProfile
    }
}));

vi.mock('@/state/runtimeStore', () => {
    const useRuntimeStore = Object.assign(vi.fn(), {
        getState: () => ({
            auth: {
                currentUserId: mocks.currentUserId,
                currentUserEndpoint: 'https://api.vrchat.cloud/api/1',
                currentUserSnapshot: {
                    id: 'usr_self',
                    displayName: 'Stored User',
                    status: 'active'
                }
            },
            setAuthBootstrap: mocks.setAuthBootstrap
        })
    });
    return { useRuntimeStore };
});

vi.mock('@/state/vrchatConfigStore', () => ({
    useVrchatConfigStore: (selector: (state: { snapshot: null }) => unknown) =>
        selector({ snapshot: null })
}));

vi.mock('./useCurrentUserSocialStatusDialog', () => ({
    useCurrentUserSocialStatusDialog: () => ({
        dialog: { open: false },
        openDialog: vi.fn()
    })
}));

import { useUserDialogSelfActions } from './useUserDialogSelfActions';

const profile: UserDialogProfileRecord = {
    id: 'usr_self',
    displayName: 'Current User',
    allowAvatarCopying: false,
    bio: 'Old bio',
    bioLinks: ['https://old.example'],
    pronouns: 'they/them',
    tags: ['language_en']
};

function renderActions(currentProfile: UserDialogProfileRecord = profile) {
    const actionStatusRef = { current: 'idle' };
    const setActionStatus = vi.fn();
    const setBaseProfile = vi.fn();
    const hook = renderHook(() =>
        useUserDialogSelfActions({
            profile: currentProfile,
            isCurrentUser: true,
            currentUserId: 'usr_self',
            currentUserSnapshot: profile,
            currentEndpoint: 'https://api.vrchat.cloud/api/1',
            baseProfile: currentProfile,
            setBaseProfile,
            actionStatusRef,
            setActionStatus
        })
    );
    return {
        ...hook,
        actionStatusRef,
        setActionStatus,
        setBaseProfile
    };
}

describe('useUserDialogSelfActions', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mocks.currentUserId = 'usr_self';
    });

    it('locks concurrent profile mutations and commits the resolved snapshot', async () => {
        let resolveUpdate!: (value: UserDialogProfileRecord) => void;
        mocks.updateCurrentUser.mockReturnValue(
            new Promise((resolve) => {
                resolveUpdate = resolve;
            })
        );
        const rendered = renderActions();

        const first = rendered.result.current.actions.toggleSelfAvatarCopying();
        const second =
            rendered.result.current.actions.toggleSelfAvatarCopying();

        expect(rendered.actionStatusRef.current).toBe('self-profile');
        expect(mocks.updateCurrentUser).toHaveBeenCalledOnce();
        await expect(second).resolves.toBeUndefined();

        const nextProfile = {
            ...profile,
            allowAvatarCopying: true
        };
        await act(async () => resolveUpdate(nextProfile));
        await first;

        expect(rendered.setBaseProfile).toHaveBeenCalled();
        expect(mocks.setAuthBootstrap).toHaveBeenCalledWith(
            expect.objectContaining({
                currentUserId: 'usr_self',
                currentUserSnapshot: expect.objectContaining({
                    allowAvatarCopying: true
                })
            })
        );
        expect(rendered.actionStatusRef.current).toBe('idle');
        expect(mocks.toastSuccess).toHaveBeenCalledWith(
            expect.objectContaining({
                type: 'success',
                title: 'dialog.user.success.avatar_cloning_setting_updated'
            })
        );
    });

    it('unlocks and preserves the displayed profile when a mutation fails', async () => {
        mocks.updateCurrentUser.mockRejectedValue(new Error('update failed'));
        const rendered = renderActions();

        await act(async () =>
            rendered.result.current.actions.toggleSelfBooping()
        );

        expect(rendered.setBaseProfile).not.toHaveBeenCalled();
        expect(mocks.setAuthBootstrap).not.toHaveBeenCalled();
        expect(rendered.actionStatusRef.current).toBe('idle');
        expect(mocks.toastError).toHaveBeenCalledWith(
            expect.objectContaining({ type: 'error', title: 'update failed' })
        );
    });

    it.each(['userIcon', 'profilePicOverride'] as const)(
        'reads canonical media after updating %s',
        async (fieldName) => {
            mocks.updateCurrentUser.mockResolvedValue({
                ...profile,
                [fieldName]: 'optimistic'
            });
            mocks.getUserAppearanceProfile.mockResolvedValue({
                id: 'usr_self',
                userIcon: 'https://image/file_icon/4',
                bannerCustomUrl: 'https://image/file_banner/3',
                iconUrl: 'https://image/icon/128'
            });
            const rendered = renderActions();
            await act(async () =>
                rendered.result.current.actions.setSelfProfileMediaField(
                    fieldName,
                    'file_new'
                )
            );
            expect(mocks.updateCurrentUser).toHaveBeenCalledWith({
                userId: 'usr_self',
                params: {
                    [fieldName]:
                        'https://api.vrchat.cloud/api/1/file/file_new/1'
                }
            });
            expect(mocks.getUserAppearanceProfile).toHaveBeenCalledWith({
                userId: 'usr_self',
                asSelf: true
            });
            expect(rendered.setBaseProfile).toHaveBeenCalledWith(
                expect.objectContaining({
                    userIcon: 'https://image/file_icon/4',
                    bannerCustomUrl: 'https://image/file_banner/3',
                    iconUrl: 'https://image/icon/128'
                })
            );
            expect(mocks.setAuthBootstrap).toHaveBeenCalledWith(
                expect.objectContaining({
                    currentUserSnapshot: expect.objectContaining({
                        userIcon: 'https://image/file_icon/4',
                        bannerCustomUrl: 'https://image/file_banner/3',
                        iconUrl: 'https://image/icon/128'
                    })
                })
            );
        }
    );

    it('uses bannerCustomUrl for no-op detection and clears omitted media after rereading', async () => {
        const rendered = renderActions({
            ...profile,
            userIcon: 'https://image/file_icon/1',
            bannerCustomUrl: 'https://image/file_banner/3'
        });
        await act(async () =>
            rendered.result.current.actions.setSelfProfileMediaField(
                'profilePicOverride',
                'file_banner'
            )
        );
        expect(mocks.updateCurrentUser).not.toHaveBeenCalled();
        mocks.updateCurrentUser.mockResolvedValue(profile);
        mocks.getUserAppearanceProfile.mockResolvedValue({ id: 'usr_self' });
        await act(async () =>
            rendered.result.current.actions.setSelfProfileMediaField(
                'profilePicOverride',
                ''
            )
        );
        expect(mocks.updateCurrentUser).toHaveBeenCalledWith({
            userId: 'usr_self',
            params: { profilePicOverride: '' }
        });
        expect(rendered.setBaseProfile).toHaveBeenCalledWith(
            expect.objectContaining({ userIcon: '', bannerCustomUrl: '' })
        );
    });

    it('preserves the displayed media and reports a failed profile reread', async () => {
        mocks.updateCurrentUser.mockResolvedValue(profile);
        mocks.getUserAppearanceProfile.mockRejectedValue(
            new Error('refresh failed')
        );
        const rendered = renderActions();
        await act(async () =>
            rendered.result.current.actions.setSelfProfileMediaField(
                'userIcon',
                'file_new'
            )
        );
        expect(rendered.setBaseProfile).not.toHaveBeenCalled();
        expect(mocks.setAuthBootstrap).not.toHaveBeenCalled();
        expect(mocks.toastSuccess).not.toHaveBeenCalled();
        expect(mocks.toastError).toHaveBeenCalledWith(
            expect.objectContaining({ title: 'refresh failed' })
        );
    });

    it('does not apply a media reread after switching accounts', async () => {
        mocks.updateCurrentUser.mockResolvedValue(profile);
        mocks.getUserAppearanceProfile.mockImplementation(async () => {
            mocks.currentUserId = 'usr_other';
            return { id: 'usr_self', userIcon: 'https://image/file_new/1' };
        });
        const rendered = renderActions();
        await act(async () =>
            rendered.result.current.actions.setSelfProfileMediaField(
                'userIcon',
                'file_new'
            )
        );
        expect(rendered.setBaseProfile).not.toHaveBeenCalled();
        expect(mocks.setAuthBootstrap).not.toHaveBeenCalled();
        expect(mocks.toastSuccess).not.toHaveBeenCalled();
    });

    it('applies profile fields and language removals before additions', async () => {
        mocks.updateCurrentUser.mockResolvedValue({
            ...profile,
            bio: 'New bio'
        });
        mocks.removeTags.mockResolvedValue({
            ...profile,
            bio: 'New bio',
            tags: []
        });
        mocks.addTags.mockResolvedValue({
            ...profile,
            bio: 'New bio',
            tags: ['language_ja']
        });
        const rendered = renderActions();

        act(() => rendered.result.current.actions.editSelfProfileDetails());
        act(() =>
            rendered.result.current.profileDetailsDialog.setDraft({
                languageKeys: ['ja'],
                bio: 'New bio',
                bioLinks: ['https://new.example'],
                pronouns: 'she/her'
            })
        );
        await act(async () =>
            rendered.result.current.profileDetailsDialog.onSave()
        );

        expect(mocks.updateCurrentUser).toHaveBeenCalledWith({
            userId: 'usr_self',
            params: {
                bio: 'New bio',
                bioLinks: ['https://new.example'],
                pronouns: 'she/her'
            }
        });
        expect(mocks.removeTags).toHaveBeenCalledWith({
            userId: 'usr_self',
            tags: ['language_en']
        });
        expect(mocks.addTags).toHaveBeenCalledWith({
            userId: 'usr_self',
            tags: ['language_ja']
        });
        expect(
            mocks.updateCurrentUser.mock.invocationCallOrder[0]
        ).toBeLessThan(mocks.removeTags.mock.invocationCallOrder[0]);
        expect(mocks.removeTags.mock.invocationCallOrder[0]).toBeLessThan(
            mocks.addTags.mock.invocationCallOrder[0]
        );
        expect(rendered.result.current.profileDetailsDialog.open).toBe(false);
        expect(rendered.actionStatusRef.current).toBe('idle');
    });
});
