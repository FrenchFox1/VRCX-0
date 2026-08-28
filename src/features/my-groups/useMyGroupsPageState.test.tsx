// @vitest-environment jsdom

import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const commandMocks = vi.hoisted(() => ({
    appVrchatGroupOrderGet: vi.fn(),
    appVrchatGroupOrderSet: vi.fn()
}));

const repositoryMocks = vi.hoisted(() => ({
    getUserGroups: vi.fn()
}));

const runtimeState = vi.hoisted(() => ({
    auth: {
        currentUserId: 'usr_self'
    },
    gameState: {
        isGameRunning: false
    },
    hostCapabilities: {
        registryPrefs: {
            available: true,
            reason: ''
        }
    }
}));

const toastMocks = vi.hoisted(() => ({
    error: vi.fn()
}));

const translationMocks = vi.hoisted(() => ({
    t: (key: string) => key
}));

vi.mock('@/platform/tauri/bindings', () => ({
    commands: commandMocks
}));
vi.mock('@/repositories/groupProfileRepository', () => ({
    default: repositoryMocks
}));
vi.mock('@/state/runtimeStore', () => ({
    useRuntimeStore: (selector: (state: typeof runtimeState) => unknown) =>
        selector(runtimeState)
}));
vi.mock('react-i18next', () => ({
    useTranslation: () => ({
        t: translationMocks.t
    })
}));
vi.mock('sonner', () => ({
    toast: toastMocks
}));

import { groupIdForRow } from '@/components/dialogs/user-dialog/userDialogGroupRows';

import { useMyGroupsPageState } from './useMyGroupsPageState';

const groups = [
    { id: 'grp_a', name: 'Alpha' },
    { id: 'grp_b', name: 'Beta' }
];

describe('useMyGroupsPageState', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        runtimeState.auth.currentUserId = 'usr_self';
        runtimeState.gameState.isGameRunning = false;
        runtimeState.hostCapabilities.registryPrefs.available = true;
        runtimeState.hostCapabilities.registryPrefs.reason = '';
        repositoryMocks.getUserGroups.mockResolvedValue(groups);
        commandMocks.appVrchatGroupOrderGet.mockResolvedValue([
            'grp_b',
            'grp_a'
        ]);
        commandMocks.appVrchatGroupOrderSet.mockResolvedValue(true);
    });

    afterEach(cleanup);

    it('shows groups in the in-game order by default', async () => {
        const { result } = renderHook(() => useMyGroupsPageState());

        await waitFor(() => {
            expect(result.current.status).toBe('ready');
            expect(result.current.visibleGroups.map(groupIdForRow)).toEqual([
                'grp_b',
                'grp_a'
            ]);
        });

        expect(result.current.sort).toBe('inGame');
    });

    it('adopts the in-game order when registry capability finishes loading', async () => {
        runtimeState.hostCapabilities.registryPrefs.available = false;
        const { result, rerender } = renderHook(() => useMyGroupsPageState());

        await waitFor(() => {
            expect(result.current.status).toBe('ready');
        });
        expect(result.current.sort).toBe('alphabetical');

        runtimeState.hostCapabilities.registryPrefs.available = true;
        rerender();

        await waitFor(() => {
            expect(result.current.sort).toBe('inGame');
            expect(result.current.visibleGroups.map(groupIdForRow)).toEqual([
                'grp_b',
                'grp_a'
            ]);
        });
    });

    it('uses edit mode as the only reorder mode', async () => {
        const { result } = renderHook(() => useMyGroupsPageState());

        await waitFor(() => {
            expect(result.current.status).toBe('ready');
        });
        expect(result.current.orderEditable).toBe(false);

        act(() => result.current.enterEditMode());
        expect(result.current.orderEditable).toBe(true);

        act(() => result.current.exitEditMode());
        expect(result.current.orderEditable).toBe(false);
    });

    it('persists the order produced by a completed drag', async () => {
        commandMocks.appVrchatGroupOrderGet.mockResolvedValue([
            'grp_a',
            'grp_b'
        ]);
        const { result } = renderHook(() => useMyGroupsPageState());

        await waitFor(() => {
            expect(result.current.visibleGroups.map(groupIdForRow)).toEqual([
                'grp_a',
                'grp_b'
            ]);
        });

        act(() => result.current.enterEditMode());
        await act(async () => {
            await result.current.moveGroup('grp_b', 'grp_a');
        });

        expect(commandMocks.appVrchatGroupOrderSet).toHaveBeenCalledWith([
            'grp_b',
            'grp_a'
        ]);
        expect(result.current.visibleGroups.map(groupIdForRow)).toEqual([
            'grp_b',
            'grp_a'
        ]);
    });
});
