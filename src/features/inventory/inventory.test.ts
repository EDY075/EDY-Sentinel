import { describe, expect, it } from 'vitest'
import type { InstalledSoftware } from '../../types/inventory'
import { filterInventory, formatInstallDate } from './inventory'

const sample: InstalledSoftware = {
  softwareId: 'software-1', displayName: 'Example Editor', displayVersion: '2.4', publisher: 'Example Corp',
  architecture: 'x64', installScope: 'machine', sources: ['windows_registry_machine_64'],
  registryIdentities: ['HKLM\\64\\Uninstall\\Example'], normalizedIdentity: {
    vendor: 'Example Corp', product: 'Unresolved', version: '2.4', architecture: 'x64',
    installScope: 'machine', status: 'unresolved',
  }, firstSeenAt: '2026-08-16T12:00:00Z', lastSeenAt: '2026-08-16T12:00:00Z', observationCount: 1, active: true,
}

describe('software inventory presentation', () => {
  it('filters by exact scope and conservative identity state', () => {
    expect(filterInventory([sample], '', 'machine')).toHaveLength(1)
    expect(filterInventory([sample], '', 'user')).toHaveLength(0)
    expect(filterInventory([sample], '', 'unresolved')).toHaveLength(1)
  })

  it('searches raw presentation fields without inferring identity', () => {
    expect(filterInventory([sample], 'Example Corp', 'all')).toHaveLength(1)
    expect(filterInventory([sample], 'invented product', 'all')).toHaveLength(0)
  })

  it('localizes compact Windows dates and preserves unknown formats', () => {
    expect(formatInstallDate('20260816', 'en', 'Unavailable')).toContain('2026')
    expect(formatInstallDate('20269999', 'en', 'Unavailable')).toBe('20269999')
    expect(formatInstallDate('vendor-specific', 'en', 'Unavailable')).toBe('vendor-specific')
  })
})
