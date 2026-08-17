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

export type VulnerabilityCoverageStatus =
  | 'complete'
  | 'limited'
  | 'updating'
  | 'unavailable'
  | 'not_applicable'

export interface VulnerabilityCoverage {
  status: VulnerabilityCoverageStatus
  basis: 'software_record_proxy'
  totalSoftware: number
  eligibleSoftware: number
  resolvedEligible: number
  ambiguousEligible: number
  unresolvedEligible: number
  notMappable: number
  pendingEvaluation: number
}

export interface VulnerabilityCvssBands {
  unrated: number
  low: number
  medium: number
  high: number
  critical: number
}

export interface ProductVulnerabilityRisk {
  productRiskKey: string
  canonicalVendor: string
  canonicalProduct: string
  displayNames: string[]
  softwareIds: string[]
  installedVersions: string[]
  confirmedCveIds: string[]
  possibleCveIds: string[]
  confirmedCount: number
  possibleCount: number
  cvssBands: VulnerabilityCvssBands
  highestCvss?: number
  confirmedKevCount: number
  severityAnchor: number
  marginalBreadth: number
  kevBoost: number
  uncappedImpact: number
  impact: number
  matchingEngineVersions: number[]
  identityResolverVersions: number[]
  nvdSourceVersions: string[]
  kevSourceVersions: string[]
  evaluatedAt: string
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
  detectionPenalty: number
  vulnerabilityPenalty: number
  vulnerabilityCoverage: VulnerabilityCoverage
  productVulnerabilityRisks: ProductVulnerabilityRisk[]
  reason?: string
}
