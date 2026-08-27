import { useCallback, useSyncExternalStore } from 'react';

import { normalizeLanguageCode } from '@/localization/locales';
import { tauriClient } from '@/platform/tauri/client';
import { setWindowTheme, type WindowTheme } from '@/platform/tauri/webview';
import {
    DEFAULT_THEME_COLOR_KEY,
    THEME_COLOR_STYLE_PROPERTIES
} from '@/shared/constants/themes';
import {
    normalizeThemeColor,
    resolveThemeColorConfig
} from '@/shared/utils/themeColor';
import { useShellStore, type ThemeMode } from '@/state/shellStore';

type ResolvedThemeMode = 'light' | 'dark';
type AppFontPreferenceInput = {
    fontFamily?: string;
    customFontFamily?: string;
    cjkFontPack?: string;
    locale?: string;
};
type ZoomLevelInput = string | number | null | undefined;

const NATIVE_THEME_VALUES: Readonly<Record<ThemeMode, WindowTheme | null>> =
    Object.freeze({
        system: null,
        light: 'light',
        dark: 'dark'
    });
let nativeThemeSyncQueue: Promise<void> = Promise.resolve();
let themeApplySequence = 0;
export const DEFAULT_ZOOM_LEVEL = 100;
export const MIN_ZOOM_LEVEL = 30;
export const MAX_ZOOM_LEVEL = 300;
export const ZOOM_STEP = 5;
export const COMMUNITY_THEME_FIXED_THEME_MODE: ThemeMode = 'dark';
const APP_FONT_STYLE_ATTR = 'data-vrcx-app-font';
const APP_CJK_FONT_STYLE_ATTR = 'data-vrcx-cjk-font';
const COMMUNITY_THEME_APPEARANCE_ATTR =
    'data-vrcx-0-community-theme-appearance';

export const APP_FONT_DEFAULT_KEY = 'geist';
export const APP_CJK_FONT_PACK_DEFAULT_KEY = 'noto';
const GOOGLE_NOTO_CJK_FONT_IMPORT =
    "@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@100..900&family=Noto+Sans+KR:wght@100..900&family=Noto+Sans+SC:wght@100..900&family=Noto+Sans+TC:wght@100..900&display=swap');";
const GOOGLE_NOTO_SANS_JP_FONTS = Object.freeze(["'Noto Sans JP'"]);
const GOOGLE_NOTO_SANS_SC_FONTS = Object.freeze(["'Noto Sans SC'"]);
const GOOGLE_NOTO_SANS_TC_FONTS = Object.freeze(["'Noto Sans TC'"]);
const GOOGLE_NOTO_SANS_KR_FONTS = Object.freeze(["'Noto Sans KR'"]);
const MACOS_SYSTEM_CJK_FONT_STACKS = Object.freeze({
    ja: Object.freeze(["'Hiragino Sans'", "'Hiragino Kaku Gothic ProN'"]),
    'zh-CN': Object.freeze(["'PingFang SC'", "'Hiragino Sans GB'"]),
    'zh-TW': Object.freeze(["'PingFang TC'", "'PingFang HK'"]),
    ko: Object.freeze(["'Apple SD Gothic Neo'"]),
    default: Object.freeze([])
});
const CONFIGURABLE_CJK_FONT_LOCALES = new Set(['ja', 'ko', 'zh-CN', 'zh-TW']);

export const APP_FONT_CONFIG = Object.freeze({
    inter: {
        cssName: "'Inter Variable', 'Inter'",
        cssImport:
            "@import url('https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap');"
    },
    noto_sans: {
        cssName: "'Noto Sans'",
        cssImport:
            "@import url('https://fonts.googleapis.com/css2?family=Noto+Sans:ital,wght@0,100..900;1,100..900&display=swap');"
    },
    geist: {
        cssName: "'Geist Variable', 'Geist'",
        cssImport: null
    },
    nunito_sans: {
        cssName: "'Nunito Sans'",
        cssImport:
            "@import url('https://fonts.googleapis.com/css2?family=Nunito+Sans:ital,opsz,wght@0,6..12,200..1000;1,6..12,200..1000&display=swap');"
    },
    ibm_plex_sans: {
        cssName: "'IBM Plex Sans'",
        cssImport:
            "@import url('https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:ital,wght@0,100..700;1,100..700&display=swap');"
    },
    jetbrains_mono: {
        cssName: "'JetBrains Mono'",
        cssImport:
            "@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800&display=swap');"
    },
    fantasque_sans_mono: {
        cssName: "'Fantasque Sans Mono'",
        cssImport:
            "@import url('https://fonts.cdnfonts.com/css/fantasque-sans-mono');"
    },
    system_ui: {
        cssName: 'system-ui',
        cssImport: null
    },
    custom: {
        cssName: '',
        cssImport: null
    }
});

export const APP_CJK_FONT_PACK_CONFIG = Object.freeze({
    noto: {
        cssNames: Object.freeze([]),
        cssImport: null
    },
    puhuiti: {
        cssNames: Object.freeze([
            "'PHT Sans SC'",
            "'PHT Sans TC'",
            "'PHT Sans JP'",
            "'PHT Sans KR'"
        ]),
        cssImport: [
            '/* Simplified Chinese */',
            "@font-face { font-family: 'PHT Sans SC'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/sc/phtsansSC-Regular.woff2') format('woff2'); font-weight: 400; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans SC'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/sc/phtsansSC-Medium.woff2') format('woff2'); font-weight: 500; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans SC'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/sc/phtsansSC-SemiBold.woff2') format('woff2'); font-weight: 600; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans SC'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/sc/phtsansSC-Bold.woff2') format('woff2'); font-weight: 700; font-display: swap; }",
            '/* Traditional Chinese */',
            "@font-face { font-family: 'PHT Sans TC'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/tc/phtsansTC-55.woff2') format('woff2'); font-weight: 400; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans TC'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/tc/phtsansTC-75.woff2') format('woff2'); font-weight: 600; font-display: swap; }",
            '/* Japanese */',
            "@font-face { font-family: 'PHT Sans JP'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/jp/phtsansJP-Regular.woff2') format('woff2'); font-weight: 400; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans JP'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/jp/phtsansJP-Medium.woff2') format('woff2'); font-weight: 500; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans JP'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/jp/phtsansJP-Bold.woff2') format('woff2'); font-weight: 700; font-display: swap; }",
            '/* Korean */',
            "@font-face { font-family: 'PHT Sans KR'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/kr/phtsansKR-Regular.woff2') format('woff2'); font-weight: 400; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans KR'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/kr/phtsansKR-Medium.woff2') format('woff2'); font-weight: 500; font-display: swap; }",
            "@font-face { font-family: 'PHT Sans KR'; src: url('https://cdn.jsdelivr.net/gh/map1en/pht@1.0.0/kr/phtsansKR-Bold.woff2') format('woff2'); font-weight: 700; font-display: swap; }"
        ].join('\n')
    },
    system: {
        cssNames: Object.freeze([]),
        cssImport: null
    }
});

export const APP_FONT_FAMILIES = Object.freeze(Object.keys(APP_FONT_CONFIG));
export const APP_CJK_FONT_PACKS = Object.freeze(
    Object.keys(APP_CJK_FONT_PACK_CONFIG)
);

type AppFontKey = keyof typeof APP_FONT_CONFIG;
type AppCjkFontPackKey = keyof typeof APP_CJK_FONT_PACK_CONFIG;
type ThemeColorStyleToken = keyof typeof THEME_COLOR_STYLE_PROPERTIES;

function isAppFontKey(value: string): value is AppFontKey {
    return Object.prototype.hasOwnProperty.call(APP_FONT_CONFIG, value);
}

function isAppCjkFontPackKey(value: string): value is AppCjkFontPackKey {
    return Object.prototype.hasOwnProperty.call(
        APP_CJK_FONT_PACK_CONFIG,
        value
    );
}

const THEME_COLOR_STYLE_ENTRIES: Array<[ThemeColorStyleToken, string]> = [
    ['primary', THEME_COLOR_STYLE_PROPERTIES.primary],
    ['primaryDark', THEME_COLOR_STYLE_PROPERTIES.primaryDark],
    ['foreground', THEME_COLOR_STYLE_PROPERTIES.foreground],
    ['foregroundDark', THEME_COLOR_STYLE_PROPERTIES.foregroundDark],
    ['ring', THEME_COLOR_STYLE_PROPERTIES.ring],
    ['ringDark', THEME_COLOR_STYLE_PROPERTIES.ringDark]
];

export function resolveThemeColor(value: string): string {
    return normalizeThemeColor(value);
}

export function resolveThemeMode(value: string): ThemeMode {
    if (value === 'midnight') {
        return 'dark';
    }

    if (value === 'system' || value === 'light' || value === 'dark') {
        return value;
    }

    return 'system';
}

export function isCommunityThemeAppearanceControlled(): boolean {
    if (typeof document === 'undefined') {
        return false;
    }

    return document.documentElement.hasAttribute(
        COMMUNITY_THEME_APPEARANCE_ATTR
    );
}

export function getCommunityThemeAppearanceThemeMode(): ThemeMode {
    if (typeof document === 'undefined') {
        return COMMUNITY_THEME_FIXED_THEME_MODE;
    }

    const value = document.documentElement.getAttribute(
        COMMUNITY_THEME_APPEARANCE_ATTR
    );
    return value === 'light' || value === 'dark'
        ? value
        : COMMUNITY_THEME_FIXED_THEME_MODE;
}

function resolveEffectiveThemeMode(themeMode: ThemeMode): ThemeMode {
    if (isCommunityThemeAppearanceControlled()) {
        return getCommunityThemeAppearanceThemeMode();
    }

    return resolveThemeMode(themeMode);
}

export function getResolvedThemeMode(themeMode: ThemeMode): ResolvedThemeMode {
    const normalized = resolveEffectiveThemeMode(themeMode);
    if (normalized === 'system') {
        return window.matchMedia?.('(prefers-color-scheme: dark)').matches
            ? 'dark'
            : 'light';
    }

    return normalized;
}

function subscribeSystemColorScheme(onChange: () => void): () => void {
    const query = window.matchMedia?.('(prefers-color-scheme: dark)');
    if (!query) {
        return () => undefined;
    }
    query.addEventListener('change', onChange);
    return () => query.removeEventListener('change', onChange);
}

export function useResolvedThemeMode(): ResolvedThemeMode {
    const themeMode = useShellStore((state) => state.themeMode);
    const readResolvedThemeMode = useCallback(
        () => getResolvedThemeMode(themeMode),
        [themeMode]
    );

    return useSyncExternalStore(
        subscribeSystemColorScheme,
        readResolvedThemeMode,
        readResolvedThemeMode
    );
}

export function normalizeZoomLevel(
    value: ZoomLevelInput,
    fallback: number = DEFAULT_ZOOM_LEVEL
): number {
    if (value === null || value === undefined || value === '') {
        return fallback;
    }

    const numericZoom = Number(value);
    if (!Number.isFinite(numericZoom)) {
        return fallback;
    }

    return Math.min(
        MAX_ZOOM_LEVEL,
        Math.max(MIN_ZOOM_LEVEL, Math.trunc(numericZoom))
    );
}

export function formatZoomPercentage(value: string | number): string {
    return `${normalizeZoomLevel(value)}%`;
}

function clearThemeColorProperties(root: HTMLElement): void {
    Object.values(THEME_COLOR_STYLE_PROPERTIES).forEach((propertyName) => {
        root.style.removeProperty(propertyName);
    });
}

export function clearThemeColorInlineProperties(): void {
    if (typeof document === 'undefined') {
        return;
    }
    clearThemeColorProperties(document.documentElement);
}

export function applyThemeColor(themeColor: string): string {
    const normalized = resolveThemeColor(themeColor);
    const theme = resolveThemeColorConfig(normalized);

    if (typeof document === 'undefined') {
        useShellStore.getState().setThemeColor(normalized);
        return normalized;
    }

    const root = document.documentElement;

    root.setAttribute('data-theme-color', normalized);
    clearThemeColorProperties(root);

    if (
        root.getAttribute('data-vrcx-0-community-theme-accent') !== 'theme' &&
        normalized !== DEFAULT_THEME_COLOR_KEY
    ) {
        THEME_COLOR_STYLE_ENTRIES.forEach(([tokenName, propertyName]) => {
            const cssValue = theme[tokenName];
            root.style.setProperty(propertyName, String(cssValue));
        });
    }

    useShellStore.getState().setThemeColor(normalized);
    return normalized;
}

function ensureDynamicStyle(
    attrName: string,
    styleKey: string,
    cssText: string | null
): void {
    if (typeof document === 'undefined') {
        return;
    }

    document.querySelectorAll(`style[${attrName}]`).forEach((styleElement) => {
        if (styleElement.getAttribute(attrName) !== styleKey) {
            styleElement.remove();
        }
    });

    if (
        !cssText ||
        document.querySelector(`style[${attrName}="${styleKey}"]`)
    ) {
        return;
    }

    const styleElement = document.createElement('style');
    styleElement.setAttribute(attrName, styleKey);
    styleElement.textContent = cssText;
    document.head.appendChild(styleElement);
}

export function normalizeAppFontFamily(value: string): AppFontKey {
    const normalized = value.trim().toLowerCase();
    return isAppFontKey(normalized) ? normalized : APP_FONT_DEFAULT_KEY;
}

export function normalizeAppCjkFontPack(value: string): AppCjkFontPackKey {
    const normalized = value.trim().toLowerCase();
    return isAppCjkFontPackKey(normalized)
        ? normalized
        : APP_CJK_FONT_PACK_DEFAULT_KEY;
}

function normalizeFontLocale(locale: string | undefined): string {
    const rawLocale = (
        locale ||
        useShellStore.getState().locale ||
        'en'
    ).trim();
    return normalizeLanguageCode(rawLocale || 'en');
}

export function supportsConfigurableCjkFontPack(locale: string): boolean {
    return CONFIGURABLE_CJK_FONT_LOCALES.has(normalizeFontLocale(locale));
}

export function resolveAppCjkFontPackForLocale(
    cjkFontPack: string,
    locale: string
): AppCjkFontPackKey {
    const normalizedCjk = normalizeAppCjkFontPack(cjkFontPack);
    return supportsConfigurableCjkFontPack(locale) ? normalizedCjk : 'system';
}

function getMacosSystemCjkFonts(locale: string): readonly string[] {
    switch (locale) {
        case 'ja':
            return MACOS_SYSTEM_CJK_FONT_STACKS.ja;
        case 'zh-CN':
            return MACOS_SYSTEM_CJK_FONT_STACKS['zh-CN'];
        case 'zh-TW':
            return MACOS_SYSTEM_CJK_FONT_STACKS['zh-TW'];
        case 'ko':
            return MACOS_SYSTEM_CJK_FONT_STACKS.ko;
        default:
            return MACOS_SYSTEM_CJK_FONT_STACKS.default;
    }
}

function resolveNotoCjkFontConfig(locale: string): {
    cssNames: readonly string[];
    cssImport: string | null;
    styleKey: string;
} {
    if (!supportsConfigurableCjkFontPack(locale)) {
        return {
            cssNames: [],
            cssImport: null,
            styleKey: `noto:system:${locale}`
        };
    }

    if (VRCX_0_MACOS_SYSTEM_FONTS_ENABLED) {
        return {
            cssNames: getMacosSystemCjkFonts(locale),
            cssImport: null,
            styleKey: `noto:macos:${locale}`
        };
    }

    switch (locale) {
        case 'ja':
            return {
                cssNames: GOOGLE_NOTO_SANS_JP_FONTS,
                cssImport: GOOGLE_NOTO_CJK_FONT_IMPORT,
                styleKey: 'noto:google:ja'
            };
        case 'zh-TW':
            return {
                cssNames: GOOGLE_NOTO_SANS_TC_FONTS,
                cssImport: GOOGLE_NOTO_CJK_FONT_IMPORT,
                styleKey: 'noto:google:zh-TW'
            };
        case 'ko':
            return {
                cssNames: GOOGLE_NOTO_SANS_KR_FONTS,
                cssImport: GOOGLE_NOTO_CJK_FONT_IMPORT,
                styleKey: 'noto:google:ko'
            };
        case 'zh-CN':
        default:
            return {
                cssNames: GOOGLE_NOTO_SANS_SC_FONTS,
                cssImport: GOOGLE_NOTO_CJK_FONT_IMPORT,
                styleKey: 'noto:google:zh-CN'
            };
    }
}

function resolveCjkFontConfig(
    normalizedCjk: AppCjkFontPackKey,
    locale: string
): {
    cssNames: readonly string[];
    cssImport: string | null;
    styleKey: string;
} {
    const effectiveCjk = resolveAppCjkFontPackForLocale(normalizedCjk, locale);

    if (effectiveCjk === 'noto') {
        return resolveNotoCjkFontConfig(locale);
    }

    const cjkConfig = APP_CJK_FONT_PACK_CONFIG[effectiveCjk];
    return {
        cssNames: Array.isArray(cjkConfig.cssNames) ? cjkConfig.cssNames : [],
        cssImport: cjkConfig.cssImport,
        styleKey: effectiveCjk
    };
}

export function applyAppFontPreferences({
    fontFamily = APP_FONT_DEFAULT_KEY,
    customFontFamily = '',
    cjkFontPack = APP_CJK_FONT_PACK_DEFAULT_KEY,
    locale
}: AppFontPreferenceInput = {}) {
    const normalizedFont = normalizeAppFontFamily(fontFamily);
    const normalizedCjk = normalizeAppCjkFontPack(cjkFontPack);
    const normalizedLocale = normalizeFontLocale(locale);
    const useMacosSystemFonts = VRCX_0_MACOS_SYSTEM_FONTS_ENABLED;
    const effectiveFont = useMacosSystemFonts ? 'system_ui' : normalizedFont;
    const fontConfig = APP_FONT_CONFIG[effectiveFont];

    if (effectiveFont === 'custom') {
        const stack =
            String(customFontFamily || '').trim() ||
            `${APP_FONT_CONFIG[APP_FONT_DEFAULT_KEY].cssName}, system-ui`;
        ensureDynamicStyle(APP_FONT_STYLE_ATTR, 'custom', null);
        ensureDynamicStyle(APP_CJK_FONT_STYLE_ATTR, 'custom', null);
        document.documentElement.style.setProperty(
            '--vrcx-app-font-family',
            stack
        );
        return {
            fontFamily: normalizedFont,
            customFontFamily,
            cjkFontPack: normalizedCjk
        };
    }

    const cjkConfig = useMacosSystemFonts
        ? resolveNotoCjkFontConfig(normalizedLocale)
        : resolveCjkFontConfig(normalizedCjk, normalizedLocale);
    const westernFont = fontConfig.cssName;

    ensureDynamicStyle(
        APP_FONT_STYLE_ATTR,
        effectiveFont,
        fontConfig.cssImport
    );
    ensureDynamicStyle(
        APP_CJK_FONT_STYLE_ATTR,
        cjkConfig.styleKey,
        cjkConfig.cssImport
    );

    document.documentElement.style.setProperty(
        '--vrcx-app-font-family',
        [westernFont, ...cjkConfig.cssNames, 'system-ui']
            .filter(Boolean)
            .join(', ')
    );

    return {
        fontFamily: normalizedFont,
        customFontFamily,
        cjkFontPack: normalizedCjk
    };
}

export function syncNativeTheme(themeMode: ThemeMode): Promise<void> {
    const normalized = resolveEffectiveThemeMode(themeMode);
    const sync = nativeThemeSyncQueue.then(async () => {
        await setWindowTheme(NATIVE_THEME_VALUES[normalized]);
    });

    nativeThemeSyncQueue = sync.catch(() => undefined);
    return sync;
}

export async function applyThemeMode(themeMode: string): Promise<void> {
    const sequence = ++themeApplySequence;
    const normalized = resolveThemeMode(themeMode);
    const effectiveThemeMode = resolveEffectiveThemeMode(normalized);

    if (effectiveThemeMode === 'system') {
        await syncNativeTheme(effectiveThemeMode);
        if (sequence !== themeApplySequence) {
            return;
        }
    }

    const resolvedTheme = getResolvedThemeMode(effectiveThemeMode);
    const shouldUseDarkClass = resolvedTheme === 'dark';

    document.documentElement.classList.toggle('dark', shouldUseDarkClass);
    document.documentElement.setAttribute('data-theme', resolvedTheme);

    useShellStore.getState().setThemeMode(effectiveThemeMode);
    if (effectiveThemeMode !== 'system') {
        await syncNativeTheme(effectiveThemeMode);
    }
}

export async function setCommunityThemeAppearanceControl(
    enabled: boolean,
    restoredThemeMode: ThemeMode = useShellStore.getState().themeMode,
    controlledThemeMode: ThemeMode = COMMUNITY_THEME_FIXED_THEME_MODE
): Promise<void> {
    if (typeof document === 'undefined') {
        return;
    }

    const root = document.documentElement;
    if (enabled) {
        const normalizedControlledThemeMode =
            resolveThemeMode(controlledThemeMode) === 'light'
                ? 'light'
                : 'dark';
        root.setAttribute(
            COMMUNITY_THEME_APPEARANCE_ATTR,
            normalizedControlledThemeMode
        );
        await applyThemeMode(normalizedControlledThemeMode);
        return;
    }

    root.removeAttribute(COMMUNITY_THEME_APPEARANCE_ATTR);
    await applyThemeMode(restoredThemeMode);
}

export async function applyZoomLevel(savedZoom: ZoomLevelInput): Promise<void> {
    if (savedZoom === null || savedZoom === undefined) {
        return;
    }

    const numericZoom = normalizeZoomLevel(savedZoom);

    useShellStore.getState().setZoomLevel(numericZoom);
    await tauriClient.webview.setZoom(Math.pow(1.2, numericZoom / 10 - 10));
}
