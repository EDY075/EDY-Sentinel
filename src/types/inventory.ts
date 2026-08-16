export interface SoftwareIdentity {
  vendor: string
  product: string
  version: string
  architecture: string
  installScope: string
  status: 'resolved' | 'unresolved'
}

export interface InstalledSoftware {
  softwareId: string
  displayName: string
  displayVersion?: string
  publisher?: string
  installLocation?: string
  installDate?: string
  architecture: 'x64' | 'x86' | 'arm64' | 'unresolved'
  installScope: 'machine' | 'user'
  sources: string[]
  registryIdentities: string[]
  productCode?: string
  normalizedIdentity: SoftwareIdentity
  firstSeenAt: string
  lastSeenAt: string
  observationCount: number
  active: boolean
}

export interface SoftwareInventorySnapshot {
  items: InstalledSoftware[]
  collectedAt: string
  durationMs: number
  rawEntryCount: number
  sourceCount: number
}

export type VulnerabilityProviderName = 'nvd' | 'cisa_kev'
export type VulnerabilityProviderState = 'idle' | 'ready' | 'updating' | 'error'

export interface VulnerabilityProviderStatus {
  provider: VulnerabilityProviderName
  status: VulnerabilityProviderState
  lastAttemptAt?: string
  lastSuccessfulSyncAt?: string
  recordCount: number
  errorCode?: string
}
