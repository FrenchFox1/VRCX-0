// @vitest-environment jsdom

import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { SavedAuthSnapshot } from '@/repositories/authRepository';
import type { AppToastOptions } from '@/services/toastService';

const mocks = vi.hoisted(() => ({
    executeAutoLogin: vi.fn(),
    toastError: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('@/services/i18nService', () => ({
    default: { t: (key: string) => key }
}));

vi.mock('@/services/toastService', () => ({
    toast: {
        add: (options: AppToastOptions) => {
            switch (options.type) {
                case 'error':
                    return mocks.toastError(options);
                default:
                    throw new Error('Unhandled toast type: ' + options.type);
            }
        }
    }
}));

vi.mock('@/services/authAutoLoginService', () => ({
    executeReactAutoLogin: mocks.executeAutoLogin
}));

vi.mock('@/state/runtimeStore', () => ({
    useRuntimeStore: (
        selector: (state: {
            shell: { backendRuntimeSnapshotHydrated: boolean };
        }) => unknown
    ) => selector({ shell: { backendRuntimeSnapshotHydrated: true } })
}));

import { useLoginAutoLogin } from './useLoginAutoLogin';

const savedCredential = {
    hasCookies: false,
    hasLoginCredentials: true,
    loginParams: {
        username: 'account'
    },
    user: { id: 'usr_1', displayName: 'User One' }
};

const snapshot: SavedAuthSnapshot = {
    autoLoginDelayEnabled: false,
    autoLoginDelaySeconds: 0,
    autoLoginReason: '',
    autoLoginStatus: 'available',
    lastUserLoggedIn: 'usr_1',
    savedCredentialsList: [savedCredential]
};

describe('useLoginAutoLogin', () => {
    beforeEach(() => {
        mocks.executeAutoLogin.mockReset();
        mocks.toastError.mockReset();
    });

    it('applies the snapshot returned by automatic login', async () => {
        const nextSnapshot: SavedAuthSnapshot = {
            ...snapshot,
            autoLoginDelayEnabled: true,
            autoLoginDelaySeconds: 5,
            autoLoginReason:
                'Saved credentials are available. Auto-login delay is 5 second(s).'
        };
        const applySnapshot = vi.fn((_value: SavedAuthSnapshot) => undefined);
        mocks.executeAutoLogin.mockResolvedValue({
            snapshot: nextSnapshot,
            status: 'success'
        });

        renderHook(() =>
            useLoginAutoLogin({
                activeSavedUserId: '',
                applySnapshot,
                databaseReady: true,
                isLoading: false,
                isSubmitting: false,
                snapshot
            })
        );

        await waitFor(() => {
            expect(applySnapshot).toHaveBeenCalledWith(nextSnapshot);
        });
    });

    it('aborts the active automatic login when manual interaction starts', async () => {
        mocks.executeAutoLogin.mockImplementation(
            (
                _snapshot: SavedAuthSnapshot,
                { signal }: { signal: AbortSignal }
            ) =>
                new Promise((resolve) => {
                    signal.addEventListener('abort', () => {
                        resolve({ snapshot, status: 'cancelled' });
                    });
                })
        );
        const applySnapshot = vi.fn((_value: SavedAuthSnapshot) => undefined);
        const { result } = renderHook(() =>
            useLoginAutoLogin({
                activeSavedUserId: '',
                applySnapshot,
                databaseReady: true,
                isLoading: false,
                isSubmitting: false,
                snapshot
            })
        );

        await waitFor(() => {
            expect(mocks.executeAutoLogin).toHaveBeenCalledTimes(1);
        });
        act(() => {
            result.current.cancelPendingAutoLogin();
        });

        expect(mocks.executeAutoLogin.mock.calls[0]?.[1]).toMatchObject({
            signal: { aborted: true }
        });
    });

    it('keeps an unexpected automatic-login failure visible until dismissed', async () => {
        mocks.executeAutoLogin.mockRejectedValue(new Error('Login failed'));

        renderHook(() =>
            useLoginAutoLogin({
                activeSavedUserId: '',
                applySnapshot: vi.fn(),
                databaseReady: true,
                isLoading: false,
                isSubmitting: false,
                snapshot
            })
        );

        await waitFor(() => {
            expect(mocks.toastError).toHaveBeenCalledWith(
                expect.objectContaining({
                    type: 'error',
                    title: 'Login failed',
                    timeout: 0,
                    data: expect.objectContaining({ closeButton: true })
                })
            );
        });
    });
});
