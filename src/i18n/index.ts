import i18next from 'i18next'
import { initReactI18next } from 'react-i18next'
import { loadLanguage, loadSystemLocale, persistLanguage } from '../lib/tauri'
import {
  DEFAULT_LANGUAGE,
  SUPPORTED_LANGUAGES,
  detectInitialLanguage,
  normalizeLanguage,
  readNavigatorLanguages,
  type SupportedLanguage,
} from './language'
import { namespaces, resources } from './resources'

export const i18n = i18next

export const I18N_INIT_OPTIONS = {
  resources,
  supportedLngs: SUPPORTED_LANGUAGES,
  fallbackLng: DEFAULT_LANGUAGE,
  defaultNS: 'common',
  ns: namespaces,
  load: 'currentOnly' as const,
  nonExplicitSupportedLngs: false,
  returnNull: false,
  interpolation: { escapeValue: false },
}

function applyDocumentLanguage(language: SupportedLanguage): void {
  if (typeof document !== 'undefined') document.documentElement.lang = language
}

export async function initializeI18n(): Promise<SupportedLanguage> {
  const savedLanguage = await loadLanguage().catch(() => null)
  const systemLocale = savedLanguage ? null : await loadSystemLocale().catch(() => null)
  const detectedLanguages = [systemLocale, ...readNavigatorLanguages()].filter((value): value is string => Boolean(value))
  const language = detectInitialLanguage(savedLanguage, detectedLanguages)
  if (!i18n.isInitialized) {
    await i18n.use(initReactI18next).init({ ...I18N_INIT_OPTIONS, lng: language })
  } else {
    await i18n.changeLanguage(language)
  }
  applyDocumentLanguage(language)
  return language
}

export function getActiveLanguage(): SupportedLanguage {
  return normalizeLanguage(i18n.resolvedLanguage ?? i18n.language) ?? DEFAULT_LANGUAGE
}

export async function changeLanguage(language: SupportedLanguage): Promise<void> {
  const previousLanguage = getActiveLanguage()
  await i18n.changeLanguage(language)
  applyDocumentLanguage(language)
  try {
    await persistLanguage(language)
  } catch (error) {
    await i18n.changeLanguage(previousLanguage)
    applyDocumentLanguage(previousLanguage)
    throw error
  }
}

export {
  DEFAULT_LANGUAGE,
  SUPPORTED_LANGUAGES,
  detectInitialLanguage,
  normalizeLanguage,
  readNavigatorLanguages,
}
export type { SupportedLanguage }
export { formatDate, formatDateTime, formatNumber, formatRelativeTime } from './format'
export { flattenResourceKeys, namespaces, resources } from './resources'
