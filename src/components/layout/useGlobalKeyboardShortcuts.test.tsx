// @vitest-environment jsdom

import { act, cleanup, renderHook } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter } from 'react-router';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { useRuntimeStore } from '@/state/runtimeStore';

import { useGlobalKeyboardShortcuts } from './useGlobalKeyboardShortcuts';

function wrapper({ children }: { children: ReactNode }) {
    return <MemoryRouter>{children}</MemoryRouter>;
}

describe('useGlobalKeyboardShortcuts', () => {
    beforeEach(() => {
        useRuntimeStore.getState().resetRuntimeState();
    });

    afterEach(() => {
        cleanup();
    });

    it('toggles the keyboard shortcuts dialog with Ctrl+/', () => {
        renderHook(() => useGlobalKeyboardShortcuts(), { wrapper });

        const openShortcut = new KeyboardEvent('keydown', {
            cancelable: true,
            ctrlKey: true,
            key: '/'
        });
        act(() => window.dispatchEvent(openShortcut));

        expect(openShortcut.defaultPrevented).toBe(true);
        expect(
            useRuntimeStore.getState().systemHosts.keyboardShortcutsOpen
        ).toBe(true);

        const closeShortcut = new KeyboardEvent('keydown', {
            cancelable: true,
            ctrlKey: true,
            key: '/'
        });
        act(() => window.dispatchEvent(closeShortcut));

        expect(closeShortcut.defaultPrevented).toBe(true);
        expect(
            useRuntimeStore.getState().systemHosts.keyboardShortcutsOpen
        ).toBe(false);
    });
});
