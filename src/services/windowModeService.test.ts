// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { WindowGeometry } from '@/platform/tauri/webview';

import type { WindowAnimationBounds } from './windowModeAnimation';

const mocks = vi.hoisted(() => ({
    getWindowGeometry: vi.fn<() => Promise<WindowGeometry | null>>(),
    maximizeWindow: vi.fn<() => Promise<void>>(),
    unmaximizeWindow: vi.fn<() => Promise<void>>(),
    setWindowBounds: vi.fn<(bounds: WindowAnimationBounds) => Promise<void>>(),
    setWindowPhysicalPosition: vi.fn<(x: number, y: number) => Promise<void>>(),
    setWindowSizeConstraints:
        vi.fn<(constraints: Record<string, number>) => Promise<void>>(),
    setWindowMaximizable: vi.fn<(maximizable: boolean) => Promise<void>>(),
    animateWindowBounds:
        vi.fn<
            (
                start: WindowAnimationBounds,
                end: WindowAnimationBounds,
                allowAnimation?: boolean
            ) => Promise<void>
        >()
}));

vi.mock('@/platform/tauri/client', () => ({
    tauriClient: {
        webview: mocks
    }
}));

vi.mock('@/services/shellIntegrationService', () => ({
    setTaskbarOverlayNotification: vi.fn(),
    setTrayIconNotification: vi.fn()
}));

vi.mock('./windowModeAnimation', () => ({
    animateWindowBounds: mocks.animateWindowBounds
}));

import { useShellStore } from '@/state/shellStore';

import {
    enterSidebarWindowMode,
    initializeWindowDisplayMode,
    restoreNormalWindowMode
} from './windowModeService';

const storedValues = new Map<string, string>();
Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
        clear: () => storedValues.clear(),
        getItem: (key: string) => storedValues.get(key) ?? null,
        removeItem: (key: string) => storedValues.delete(key),
        setItem: (key: string, value: string) => {
            storedValues.set(key, value);
        }
    }
});

function createGeometry(
    overrides: Partial<WindowGeometry> = {}
): WindowGeometry {
    const workArea = {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
        scaleFactor: 1
    };
    return {
        innerSize: { width: 1200, height: 800 },
        outerSize: { width: 1216, height: 838 },
        outerPosition: { x: 100, y: 100 },
        scaleFactor: 1,
        maximized: false,
        currentWorkArea: workArea,
        workAreas: [workArea],
        ...overrides
    };
}

beforeEach(() => {
    window.localStorage.clear();
    useShellStore.setState({ windowDisplayMode: 'normal' });
    Object.values(mocks).forEach((mock) => mock.mockReset());
    mocks.getWindowGeometry.mockResolvedValue(null);
    mocks.maximizeWindow.mockResolvedValue(undefined);
    mocks.unmaximizeWindow.mockResolvedValue(undefined);
    mocks.setWindowBounds.mockResolvedValue(undefined);
    mocks.setWindowPhysicalPosition.mockResolvedValue(undefined);
    mocks.setWindowSizeConstraints.mockResolvedValue(undefined);
    mocks.setWindowMaximizable.mockResolvedValue(undefined);
    mocks.animateWindowBounds.mockImplementation(async (_start, end) => {
        await mocks.setWindowBounds(end);
    });
});

describe('windowModeService', () => {
    it('captures the normal bounds and right-anchors the initial sidebar width', async () => {
        mocks.getWindowGeometry
            .mockResolvedValueOnce(createGeometry())
            .mockResolvedValueOnce(
                createGeometry({
                    innerSize: { width: 480, height: 800 },
                    outerSize: { width: 496, height: 838 }
                })
            );

        await enterSidebarWindowMode(480);

        expect(useShellStore.getState().windowDisplayMode).toBe('sidebar');
        expect(mocks.setWindowSizeConstraints).toHaveBeenCalledWith({
            minWidth: 320,
            minHeight: 240,
            maxWidth: 800
        });
        expect(mocks.setWindowBounds).toHaveBeenCalledWith({
            width: 480,
            height: 800,
            x: 820,
            y: 100
        });
        expect(mocks.setWindowMaximizable).toHaveBeenCalledWith(false);
        expect(
            JSON.parse(
                window.localStorage.getItem('vrcx-main-window-normal-bounds') ??
                    '{}'
            )
        ).toMatchObject({
            width: 1200,
            x: 100,
            y: 100
        });
    });

    it('uses the internal default width on the first titlebar entry', async () => {
        mocks.getWindowGeometry
            .mockResolvedValueOnce(createGeometry())
            .mockResolvedValueOnce(
                createGeometry({
                    innerSize: { width: 360, height: 800 },
                    outerSize: { width: 376, height: 838 },
                    outerPosition: { x: 940, y: 100 }
                })
            );

        await enterSidebarWindowMode();

        expect(mocks.setWindowBounds).toHaveBeenCalledWith({
            width: 360,
            height: 800,
            x: 940,
            y: 100
        });
    });

    it('restores the normal width while preserving the sidebar height', async () => {
        window.localStorage.setItem(
            'vrcx-main-window-normal-bounds',
            JSON.stringify({
                version: 1,
                x: 80,
                y: 60,
                width: 1100
            })
        );
        useShellStore.getState().setWindowDisplayMode('sidebar');
        mocks.getWindowGeometry
            .mockResolvedValueOnce(
                createGeometry({
                    innerSize: { width: 520, height: 800 },
                    outerSize: { width: 536, height: 838 },
                    outerPosition: { x: 800, y: 150 }
                })
            )
            .mockResolvedValueOnce(
                createGeometry({
                    innerSize: { width: 1100, height: 800 },
                    outerSize: { width: 1116, height: 838 },
                    outerPosition: { x: 80, y: 150 }
                })
            );

        await restoreNormalWindowMode();

        expect(useShellStore.getState().windowDisplayMode).toBe('normal');
        expect(mocks.setWindowSizeConstraints).toHaveBeenCalledWith({
            minWidth: 320,
            minHeight: 240
        });
        expect(mocks.setWindowBounds).toHaveBeenCalledWith({
            width: 1100,
            height: 800,
            x: 80,
            y: 150
        });
        expect(mocks.maximizeWindow).not.toHaveBeenCalled();
        expect(
            window.localStorage.getItem('vrcx-main-window-sidebar-width')
        ).toBe('520');
    });

    it('returns to sidebar mode when normal-window geometry is unavailable', async () => {
        useShellStore.getState().setWindowDisplayMode('sidebar');

        await expect(restoreNormalWindowMode()).rejects.toThrow(
            'Unable to read the current window geometry.'
        );

        expect(useShellStore.getState().windowDisplayMode).toBe('sidebar');
        expect(mocks.setWindowSizeConstraints).toHaveBeenLastCalledWith({
            minWidth: 320,
            minHeight: 240,
            maxWidth: 800
        });
    });

    it('restores captured sidebar bounds when expansion fails', async () => {
        const compactGeometry = createGeometry({
            innerSize: { width: 520, height: 800 },
            outerSize: { width: 536, height: 838 },
            outerPosition: { x: 800, y: 150 }
        });
        useShellStore.getState().setWindowDisplayMode('sidebar');
        mocks.getWindowGeometry.mockResolvedValueOnce(compactGeometry);
        mocks.animateWindowBounds.mockRejectedValueOnce(
            new Error('animation failed')
        );

        await expect(restoreNormalWindowMode()).rejects.toThrow(
            'animation failed'
        );

        expect(useShellStore.getState().windowDisplayMode).toBe('sidebar');
        expect(mocks.setWindowBounds).toHaveBeenLastCalledWith({
            width: 520,
            height: 800,
            x: 800,
            y: 150
        });
        expect(mocks.setWindowSizeConstraints).toHaveBeenLastCalledWith({
            minWidth: 320,
            minHeight: 240,
            maxWidth: 800
        });
    });

    it('reuses a previously dragged sidebar width up to the 800px limit', async () => {
        window.localStorage.setItem('vrcx-main-window-sidebar-width', '1200');
        mocks.getWindowGeometry
            .mockResolvedValueOnce(createGeometry())
            .mockResolvedValueOnce(
                createGeometry({
                    innerSize: { width: 800, height: 800 },
                    outerSize: { width: 816, height: 838 }
                })
            );

        await enterSidebarWindowMode(360);

        expect(mocks.setWindowBounds).toHaveBeenCalledWith(
            expect.objectContaining({ width: 800, height: 800 })
        );
    });

    it('rolls back the restored normal window when entering sidebar mode fails', async () => {
        const normalGeometry = createGeometry();
        mocks.getWindowGeometry
            .mockResolvedValueOnce(
                createGeometry({
                    maximized: true,
                    innerSize: { width: 1920, height: 1040 },
                    outerSize: { width: 1920, height: 1040 },
                    outerPosition: { x: 0, y: 0 }
                })
            )
            .mockResolvedValueOnce(normalGeometry);
        mocks.setWindowBounds.mockRejectedValueOnce(new Error('resize failed'));

        await expect(enterSidebarWindowMode(480)).rejects.toThrow(
            'resize failed'
        );

        expect(useShellStore.getState().windowDisplayMode).toBe('normal');
        expect(mocks.unmaximizeWindow).toHaveBeenCalledOnce();
        expect(mocks.setWindowBounds).toHaveBeenLastCalledWith({
            width: 1200,
            height: 800,
            x: 100,
            y: 100
        });
        expect(mocks.maximizeWindow).toHaveBeenCalledOnce();
    });

    it('restores the sidebar constraints when the persisted mode starts', async () => {
        useShellStore.setState({ windowDisplayMode: 'sidebar' });

        await initializeWindowDisplayMode();

        expect(mocks.setWindowMaximizable).toHaveBeenCalledWith(false);
        expect(mocks.setWindowSizeConstraints).toHaveBeenCalledWith({
            minWidth: 320,
            minHeight: 240,
            maxWidth: 800
        });
    });

    it('recovers an out-of-range startup window to the saved sidebar width', async () => {
        window.localStorage.setItem('vrcx-main-window-sidebar-width', '480');
        useShellStore.setState({ windowDisplayMode: 'sidebar' });
        mocks.getWindowGeometry.mockResolvedValueOnce(createGeometry());

        await initializeWindowDisplayMode();

        expect(mocks.setWindowBounds).toHaveBeenCalledWith({
            width: 480,
            height: 800,
            x: 820,
            y: 100
        });
    });
});
