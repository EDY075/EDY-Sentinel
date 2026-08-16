export const SUPPORTED_LANGUAGES = ['pt-BR', 'en'] as const

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number]

export const DEFAULT_LANGUAGE: SupportedLanguage = 'en'
export const LANGUAGE_STORAGE_KEY = 'edy-sentinel-language'

export function normalizeLanguage(value: string | null | undefined): SupportedLanguage | null {
  if (!value) return null
  const normalized = value.trim().replaceAll('_', '-').toLowerCase()
  if (normalized === 'pt' || normalized.startsWith('pt-')) return 'pt-BR'
  if (normalized === 'en' || normalized.startsWith('en-')) return 'en'
  return null
}

export function detectInitialLanguage(
  savedLanguage: string | null | undefined,
  navigatorLanguages: readonly string[] = [],
): SupportedLanguage {
  const saved = normalizeLanguage(savedLanguage)
  if (saved) return saved

  return normalizeLanguage(navigatorLanguages[0]) ?? DEFAULT_LANGUAGE
}

export function readNavigatorLanguages(): string[] {
  if (typeof navigator === 'undefined') return []
  return [...new Set([...navigator.languages, navigator.language].filter(Boolean))]
}
