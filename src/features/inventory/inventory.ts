import type { InstalledSoftware } from '../../types/inventory'

export type InventoryFilter = 'all' | 'machine' | 'user' | 'resolved' | 'unresolved'

export function filterInventory(rows: InstalledSoftware[], query: string, filter: InventoryFilter): InstalledSoftware[] {
  const needle = query.trim().toLocaleLowerCase()
  return rows.filter((software) => {
    if ((filter === 'machine' || filter === 'user') && software.installScope !== filter) return false
    if ((filter === 'resolved' || filter === 'unresolved') && software.normalizedIdentity.status !== filter) return false
    if (!needle) return true
    return [software.displayName, software.displayVersion, software.publisher, software.architecture, software.installScope, software.productCode, ...software.sources]
      .some((value) => String(value ?? '').toLocaleLowerCase().includes(needle))
  })
}

export function formatInstallDate(value: string | undefined, locale: string, unavailable: string): string {
  if (!value) return unavailable
  const compact = /^(\d{4})(\d{2})(\d{2})$/.exec(value)
  if (!compact) return value
  const date = new Date(Date.UTC(Number(compact[1]), Number(compact[2]) - 1, Number(compact[3])))
  if (Number.isNaN(date.getTime())
    || date.getUTCFullYear() !== Number(compact[1])
    || date.getUTCMonth() !== Number(compact[2]) - 1
    || date.getUTCDate() !== Number(compact[3])) return value
  return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeZone: 'UTC' }).format(date)
}
