export type BaselineState = 'not_initialized' | 'learning' | 'ready' | 'stale' | 'error'
export type SecurityEventStatus = 'new' | 'seen' | 'acknowledged' | 'resolved' | 'ignored'

export interface BaselineEntityCounts {
  executables: number
  processPatterns: number
  parentChildRelationships: number
  networkDestinations: number
  services: number
  networkConfigurations: number
}

export interface BaselineSummary {
  baselineId?: string
  createdAt?: string
  learningStartedAt?: string
  learningCompletedAt?: string
  version?: number
  hostId?: string
  status: BaselineState
  observationCount: number
  schemaVersion: number
  learningPeriodSeconds: number
  lastObservedAt?: string
  lastProcessingDurationMs: number
  errorMessage?: string
  entities: BaselineEntityCounts
}

export interface SecurityEvent {
  eventId: string
  eventType: string
  entityType: string
  entityKey: string
  title: string
  timestamp: string
  firstSeen: string
  lastSeen: string
  evidence: Record<string, unknown> | null
  baselineContext: Record<string, unknown> | null
  source: string
  baselineId?: string
  ruleId?: string
  confidence?: string
  status: SecurityEventStatus
  observationCount: number
  conditionActive: boolean
  schemaVersion: number
}

export interface BaselineAction {
  confirmation: string
  learningPeriodSeconds?: number
}
