export function formatBytes(bytes?: number, locale?: string, unavailable = 'Unavailable') {
  if (bytes == null || !Number.isFinite(bytes)) return unavailable
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = Math.max(0, bytes)
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit += 1 }
  return `${new Intl.NumberFormat(locale ?? 'en-US', { minimumFractionDigits: 0, maximumFractionDigits: value >= 10 || unit === 0 ? 0 : 1 }).format(value)} ${units[unit]}`
}

export function formatPercent(value?: number | null, locale?: string, calculating = 'Calculating') {
  if (value == null || !Number.isFinite(value)) return calculating
  return new Intl.NumberFormat(locale ?? 'en-US', { style: 'percent', minimumFractionDigits: 0, maximumFractionDigits: value < 10 ? 1 : 0 }).format(value / 100)
}

export function formatAge(value?: string, locale?: string, unknown = 'unknown') {
  if (!value) return unknown
  const seconds = Math.max(0, (Date.now() - new Date(value).getTime()) / 1000)
  if (!Number.isFinite(seconds)) return unknown
  const relative = new Intl.RelativeTimeFormat(locale ?? 'en-US', { numeric: 'always', style: 'short' })
  if (seconds < 60) return relative.format(-Math.round(seconds), 'second')
  if (seconds < 3_600) return relative.format(-Math.floor(seconds / 60), 'minute')
  if (seconds < 86_400) return relative.format(-Math.floor(seconds / 3_600), 'hour')
  return relative.format(-Math.floor(seconds / 86_400), 'day')
}

export function formatDateTime(value?: string, locale?: string, unavailable = 'Unavailable') {
  if (!value) return unavailable
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : new Intl.DateTimeFormat(locale ?? 'en-US', { dateStyle: 'medium', timeStyle: 'medium' }).format(date)
}

export function formatDurationFrom(value?: string, locale?: string, unavailable = 'Unavailable') {
  if (!value) return unavailable
  const seconds = Math.max(0, Math.floor((Date.now() - new Date(value).getTime()) / 1000))
  if (!Number.isFinite(seconds)) return unavailable
  const number = new Intl.NumberFormat(locale ?? 'en-US')
  const days = Math.floor(seconds / 86_400)
  const hours = Math.floor((seconds % 86_400) / 3_600)
  const minutes = Math.floor((seconds % 3_600) / 60)
  return days ? `${number.format(days)}d ${number.format(hours)}h` : hours ? `${number.format(hours)}h ${number.format(minutes)}m` : `${number.format(minutes)}m`
}

export const displayValue = (value: unknown, unavailable = 'Unavailable') => value == null || value === '' || value === 'Unavailable' ? unavailable : String(value)
