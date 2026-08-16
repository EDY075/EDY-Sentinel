import { getActiveLanguage } from './index'
import type { SupportedLanguage } from './language'

type DateValue = Date | number | string

function asDate(value: DateValue): Date {
  return value instanceof Date ? value : new Date(value)
}

export function formatNumber(
  value: number,
  options?: Intl.NumberFormatOptions,
  language: SupportedLanguage = getActiveLanguage(),
): string {
  return new Intl.NumberFormat(language, options).format(value)
}

export function formatDate(
  value: DateValue,
  options?: Intl.DateTimeFormatOptions,
  language: SupportedLanguage = getActiveLanguage(),
): string {
  return new Intl.DateTimeFormat(language, options ?? { dateStyle: 'medium' }).format(asDate(value))
}

export function formatDateTime(
  value: DateValue,
  options?: Intl.DateTimeFormatOptions,
  language: SupportedLanguage = getActiveLanguage(),
): string {
  return new Intl.DateTimeFormat(language, options ?? { dateStyle: 'medium', timeStyle: 'medium' }).format(asDate(value))
}

export function formatRelativeTime(
  value: number,
  unit: Intl.RelativeTimeFormatUnit,
  options?: Intl.RelativeTimeFormatOptions,
  language: SupportedLanguage = getActiveLanguage(),
): string {
  return new Intl.RelativeTimeFormat(language, options).format(value, unit)
}
