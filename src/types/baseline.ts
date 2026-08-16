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
  updatedAt?: string
  lastProcessingDurationMs: number
  errorCode?: string
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
  ruleVersion?: number
  confidence?: string
  status: SecurityEventStatus
  observationCount: number
  conditionActive: boolean
  schemaVersion: number
}

export interface SecurityEventCursor {
  lastSeenAt: string
  eventId: string
}

export interface SecurityEventQuery {
  statuses?: SecurityEventStatus[]
  eventTypes?: string[]
  entityType?: string
  entityKey?: string
  baselineId?: string
  from?: string
  to?: string
  cursor?: SecurityEventCursor
  limit?: number
}

export interface SecurityEventPage {
  items: SecurityEvent[]
  nextCursor?: SecurityEventCursor
  hasMore: boolean
}

export type SecurityEventTransition = 'first_observed' | 'reactivated' | 'inactive' | 'status_changed'

export interface SecurityEventHistoryCursor {
  observedAt: string
  historyId: number
}

export interface SecurityEventHistoryQuery {
  entityType: string
  entityKey: string
  eventType?: string
  baselineId?: string
  from?: string
  to?: string
  cursor?: SecurityEventHistoryCursor
  limit?: number
}

export interface SecurityEventHistoryRecord {
  historyId: number
  eventId: string
  transition: SecurityEventTransition
  observedAt: string
  recordedAt: string
  source: string
  eventType: string
  entityType: string
  entityKey: string
  baselineId?: string
  ruleId?: string
  ruleVersion?: number
  evidence: Record<string, unknown> | null
  baselineContext: Record<string, unknown> | null
  previousStatus?: SecurityEventStatus
  newStatus?: SecurityEventStatus
  eventSchemaVersion: number
  historySchemaVersion: number
  observationCount: number
}

export interface SecurityEventHistoryPage {
  items: SecurityEventHistoryRecord[]
  nextCursor?: SecurityEventHistoryCursor
  hasMore: boolean
}

export interface BaselineAction {
  confirmation: string
  learningPeriodSeconds?: number
}
