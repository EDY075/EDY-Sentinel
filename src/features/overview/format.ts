export function formatBytes(bytes?: number, locale?: string, unavailable = 'Unavailable'): string {
  if (bytes === undefined || !Number.isFinite(bytes)) return unavailable
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const value = bytes / 1024 ** exponent
  return `${new Intl.NumberFormat(locale ?? 'en-US', { minimumFractionDigits: 0, maximumFractionDigits: exponent > 2 ? 1 : 0 }).format(value)} ${units[exponent]}`
}

export function formatUptime(seconds: number, locale?: string): string {
  const number = new Intl.NumberFormat(locale ?? 'en-US')
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return days > 0 ? `${number.format(days)}d ${number.format(hours)}h ${number.format(minutes)}m` : `${number.format(hours)}h ${number.format(minutes)}m`
}

export function percent(used: number, total: number): number {
  return total > 0 ? Math.min(100, Math.round((used / total) * 100)) : 0
}
