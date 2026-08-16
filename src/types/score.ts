import type { DetectionConfidence, DetectionSeverity } from './detection'

export type SecurityScoreState = 'available' | 'limited' | 'unavailable'

export interface ScoreCoverageItem {
  component: string
  status: string
  detail: string
}

export interface ScoreBreakdownItem {
  correlationKey: string
  title: string
  severity: DetectionSeverity
  confidence: DetectionConfidence
  penalty: number
}

export interface SecurityScore {
  state: SecurityScoreState
  score?: number
  label?: string
  generatedAt?: string
  formulaVersion: number
  activeDetectionCount: number
  highestSeverity?: DetectionSeverity
  coverage: ScoreCoverageItem[]
  breakdown: ScoreBreakdownItem[]
  reason?: string
}
