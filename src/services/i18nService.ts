import { createInstance } from 'i18next';
import { initReactI18next } from 'react-i18next';

import {
    FALLBACK_LOCALE_CODE,
    fallbackLocaleMessages,
    getLoadedLocaleMessages,
    loadLocaleMessages
} from '@/localization/index';
import { normalizeLanguageCode } from '@/localization/locales';

type TimeUnitLabels = Record<string, string>;
const i18nResources = {
    [FALLBACK_LOCALE_CODE]: { translation: fallbackLocaleMessages }
};

export const i18n = createInstance();
const i18nReady = i18n.use(initReactI18next).init({
    lng: 'en',
    fallbackLng: 'en',
    ns: ['translation'],
    defaultNS: 'translation',
    resources: i18nResources,
    interpolation: {
        escapeValue: false,
        prefix: '{',
        suffix: '}'
    },
    react: {
        useSuspense: false
    },
    returnNull: false
});

export default i18n;

function isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value && typeof value === 'object');
}

function resolveMessage(messages: unknown, key: string): unknown {
    return key
        .split('.')
        .reduce(
            (current: unknown, part) =>
                isRecord(current) ? current[part] : undefined,
            messages
        );
}

function normalizeLocale(locale: unknown): string {
    return normalizeLanguageCode(locale);
}

export async function setI18nLanguage(locale: unknown): Promise<string> {
    const normalizedLocale = normalizeLocale(locale);
    await i18nReady;
    if (!i18n.hasResourceBundle(normalizedLocale, 'translation')) {
        const messages = await loadLocaleMessages(normalizedLocale);
        i18n.addResourceBundle(normalizedLocale, 'translation', messages);
    }
    await i18n.changeLanguage(normalizedLocale);
    return normalizedLocale;
}

export function getTimeUnitLabels(
    locale: unknown,
    defaultLabels: TimeUnitLabels
): TimeUnitLabels {
    const localizedMessages =
        getLoadedLocaleMessages(normalizeLocale(locale)) ?? {};
    const fallbackMessages = fallbackLocaleMessages;
    const labels: TimeUnitLabels = {};

    for (const unit of Object.keys(defaultLabels)) {
        const key = `common.time_units.${unit}`;
        const localized = resolveMessage(localizedMessages, key);
        const fallback = resolveMessage(fallbackMessages, key);
        labels[unit] =
            typeof localized === 'string'
                ? localized
                : typeof fallback === 'string'
                  ? fallback
                  : defaultLabels[unit];
    }

    return labels;
}
