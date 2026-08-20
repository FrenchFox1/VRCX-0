import { normalizePlatformError } from './errors';

type WebviewWindowLike = {
    setZoom?: (zoom: number) => Promise<void>;
    scaleFactor?: (() => Promise<number> | number) | number;
};

export type WindowResizeDirection =
    | 'North'
    | 'NorthEast'
    | 'East'
    | 'SouthEast'
    | 'South'
    | 'SouthWest'
    | 'West'
    | 'NorthWest';

export type WindowTheme = 'light' | 'dark';

type WindowLike = {
    startDragging?: () => Promise<void>;
    startResizeDragging?: (direction: WindowResizeDirection) => Promise<void>;
    minimize?: () => Promise<void>;
    toggleMaximize?: () => Promise<void>;
    close?: () => Promise<void>;
    isMaximized?: () => Promise<boolean> | boolean;
    setFocus?: () => Promise<void>;
    requestUserAttention?: (requestType: number | null) => Promise<void>;
    setTheme?: (theme: WindowTheme | null) => Promise<void>;
};

async function loadCurrentWebviewWindow() {
    try {
        const module = await import('@tauri-apps/api/webviewWindow');
        return module.getCurrentWebviewWindow;
    } catch (error) {
        throw normalizePlatformError(
            error,
            'Unable to load Tauri webviewWindow API'
        );
    }
}

async function loadWindowModule() {
    try {
        return await import('@tauri-apps/api/window');
    } catch (error) {
        throw normalizePlatformError(error, 'Unable to load Tauri window API');
    }
}

async function loadCurrentWindow() {
    const module = await loadWindowModule();
    return module.getCurrentWindow;
}

export async function getCurrentWebviewWindow(): Promise<WebviewWindowLike> {
    const getWindow = await loadCurrentWebviewWindow();
    return getWindow();
}

export async function getCurrentWindow(): Promise<WindowLike> {
    const getWindow = await loadCurrentWindow();
    return getWindow();
}

export async function setZoom(zoom: number): Promise<void> {
    const current = await getCurrentWebviewWindow();
    if (current && typeof current.setZoom === 'function') {
        return current.setZoom(zoom);
    }
    return undefined;
}

export async function getScaleFactor(): Promise<number | null> {
    const current = await getCurrentWebviewWindow();
    if (!current) {
        return null;
    }

    if (typeof current.scaleFactor === 'function') {
        return current.scaleFactor();
    }

    if (typeof current.scaleFactor === 'number') {
        return current.scaleFactor;
    }

    return null;
}

export async function startDraggingWindow(): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.startDragging === 'function') {
        return current.startDragging();
    }
    return undefined;
}

export async function startResizeDraggingWindow(
    direction: WindowResizeDirection
): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.startResizeDragging === 'function') {
        return current.startResizeDragging(direction);
    }
    return undefined;
}

export async function minimizeWindow(): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.minimize === 'function') {
        return current.minimize();
    }
    return undefined;
}

export async function toggleMaximizeWindow(): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.toggleMaximize === 'function') {
        return current.toggleMaximize();
    }
    return undefined;
}

export async function closeWindow(): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.close === 'function') {
        return current.close();
    }
    return undefined;
}

export async function focusWindow(): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.setFocus === 'function') {
        return current.setFocus();
    }
    return undefined;
}

export async function flashWindow(): Promise<void> {
    const module = await loadWindowModule();
    const current: WindowLike = module.getCurrentWindow();
    if (current && typeof current.requestUserAttention === 'function') {
        return current.requestUserAttention(
            module.UserAttentionType.Informational
        );
    }
    return undefined;
}

export async function setWindowTheme(theme: WindowTheme | null): Promise<void> {
    const current = await getCurrentWindow();
    if (current && typeof current.setTheme === 'function') {
        return current.setTheme(theme);
    }
    return undefined;
}

export async function isWindowMaximized(): Promise<boolean> {
    const current = await getCurrentWindow();
    if (current && typeof current.isMaximized === 'function') {
        return current.isMaximized();
    }
    return false;
}

export const webview = Object.freeze({
    getCurrentWebviewWindow,
    getCurrentWindow,
    setZoom,
    getScaleFactor,
    startDraggingWindow,
    startResizeDraggingWindow,
    minimizeWindow,
    toggleMaximizeWindow,
    closeWindow,
    focusWindow,
    flashWindow,
    setWindowTheme,
    isWindowMaximized
});
