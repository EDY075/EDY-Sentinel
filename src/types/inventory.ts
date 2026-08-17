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
  recordsProcessed: number
  pagesProcessed: number
  lastSuccessfulPage?: number
  syncElapsedMs?: number
  errorCode?: string
}

export type VulnerabilityMatchState = 'confirmed' | 'possible' | 'unresolved' | 'not_affected'
export type VulnerabilityConfidence = 'high' | 'medium' | 'low'
export type VulnerabilityEvaluationState = 'not_evaluated' | 'confirmed' | 'possible' | 'unresolved' | 'not_affected' | 'no_confirmed'

export interface SoftwareVulnerabilitySummary {
  softwareId: string
  evaluationState: VulnerabilityEvaluationState
  confirmedCount: number
  possibleCount: number
  unresolvedCount: number
  notAffectedCount: number
  highestCvss?: number
  kevCount: number
  lastEvaluatedAt?: string
  matchingEngineVersion?: number
}

export interface VulnerabilityEvaluationSummary {
  softwareEvaluated: number
  confirmed: number
  possible: number
  unresolved: number
  notAffected: number
  confirmedCves: number
  kevCves: number
  candidateCves: number
  durationMs: number
  matchingEngineVersion: number
  remainingQueued: number
}

export interface VulnerabilityEvidence {
  evidenceId: number
  evidenceType: string
  source: string
  observedAt: string
  details: Record<string, unknown>
}

export interface KevContext {
  vulnerabilityName: string
  dateAdded: string
  dueDate?: string
  requiredAction: string
  knownRansomwareCampaignUse?: string
}

export interface VulnerabilityMatch {
  matchId: string
  cveId: string
  matchState: VulnerabilityMatchState
  confidence: VulnerabilityConfidence
  installedVersion?: string
  cpe: string
  affectedRange: string
  comparisonResult: string
  cvssScore?: number
  cvssVersion?: string
  severity?: string
  description: string
  publishedAt: string
  lastModifiedAt: string
  references: string[]
  kev?: KevContext
  evidence: VulnerabilityEvidence[]
  lastEvaluatedAt: string
  matchingEngineVersion: number
  nvdSourceVersion?: string
  kevSourceVersion?: string
}

export interface SoftwareVulnerabilityDetail {
  summary: SoftwareVulnerabilitySummary
  matches: VulnerabilityMatch[]
}
