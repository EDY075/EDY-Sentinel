export type DetectionSeverity = 'informational' | 'low' | 'medium' | 'high' | 'critical'
export type DetectionConfidence = 'low' | 'medium' | 'high'
export type DetectionStatus = 'new' | 'investigating' | 'acknowledged' | 'resolved' | 'ignored'

export interface DetectionExplanation {
  whatHappened: string
  whyFlagged: string
  severityReason: string
  confidenceReason: string
}

export interface Detection {
  detectionId: string
  ruleId: string
  ruleVersion: number
  entityType: string
  entityKey: string
  title: string
  summary: string
  severity: DetectionSeverity
  confidence: DetectionConfidence
  status: DetectionStatus
  firstDetectedAt: string
  lastDetectedAt: string
  occurrenceCount: number
  baselineId?: string
  explanation: DetectionExplanation
  remediationGuidance: string[]
  conditionActive: boolean
  schemaVersion: number
}

export interface DetectionCursor {
  lastDetectedAt: string
  detectionId: string
}

export interface DetectionQuery {
  severities?: DetectionSeverity[]
  categories?: string[]
  statuses?: DetectionStatus[]
  ruleIds?: string[]
  entityType?: string
  entityKey?: string
  from?: string
  to?: string
  cursor?: DetectionCursor
  limit?: number
}

export interface DetectionPage {
  items: Detection[]
  nextCursor?: DetectionCursor
  hasMore: boolean
}

export interface DetectionEvidenceCursor {
  observedAt: string
  evidenceId: string
}

export interface DetectionEvidenceQuery {
  detectionId: string
  cursor?: DetectionEvidenceCursor
  limit?: number
}

export interface DetectionEvidenceRecord {
  evidenceId: string
  eventId: string
  evidenceType: string
  label: string
  value: unknown
  observedAt: string
  source: string
  eventType: string
  entityType: string
  entityKey: string
  eventSchemaVersion: number
}

export interface DetectionEvidencePage {
  items: DetectionEvidenceRecord[]
  nextCursor?: DetectionEvidenceCursor
  hasMore: boolean
}
