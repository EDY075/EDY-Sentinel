export function formatBytes(bytes?: number) {
  if (bytes == null || !Number.isFinite(bytes)) return 'Unavailable'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = Math.max(0, bytes)
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit += 1 }
  return `${value >= 10 || unit === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`
}

export function formatPercent(value?: number | null) {
  if (value == null || !Number.isFinite(value)) return 'Unavailable'
  return `${value < 10 ? value.toFixed(1) : value.toFixed(0)}%`
}

export function formatDateTime(value?: string) {
  if (!value) return 'Unavailable'
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export function formatDurationFrom(value?: string) {
  if (!value) return 'Unavailable'
  const seconds = Math.max(0, Math.floor((Date.now() - new Date(value).getTime()) / 1000))
  if (!Number.isFinite(seconds)) return 'Unavailable'
  const days = Math.floor(seconds / 86_400)
  const hours = Math.floor((seconds % 86_400) / 3_600)
  const minutes = Math.floor((seconds % 3_600) / 60)
  return days ? `${days}d ${hours}h` : hours ? `${hours}h ${minutes}m` : `${minutes}m`
}

export const displayValue = (value: unknown) => value == null || value === '' ? 'Unavailable' : String(value)
