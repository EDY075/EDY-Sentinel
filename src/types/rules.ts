import type { DetectionConfidence, DetectionSeverity } from './detection'

export type { DetectionConfidence, DetectionSeverity } from './detection'

export type RuleOperator =
  | 'equals'
  | 'not_equals'
  | 'exists'
  | 'in'
  | 'at_least'
  | 'classified_as'

export type ConditionValue = string | number | boolean | string[]

export interface RuleCondition {
  factType: string
  field: string
  operator: RuleOperator
  expected: ConditionValue
}

export interface EvidenceRequirement {
  evidenceId: string
  factType: string
  acceptedEventTypes: string[]
  minimumCount: number
  correlationFields: string[]
}

export interface RulePolicy {
  policyId: string
  version: number
  description: string
  parameters: Record<string, ConditionValue>
}

/** Declarative metadata only; evaluation remains an allowlisted Rust concern. */
export interface RuleDefinition {
  ruleId: string
  version: number
  name: string
  description: string
  category: string
  enabled: boolean
  references: string[]
  conditions: RuleCondition[]
  exclusions: RuleCondition[]
  requiredEvidence: EvidenceRequirement[]
  correlationWindowSeconds: number
  defaultSeverity: DetectionSeverity
  defaultConfidence: DetectionConfidence
  severityPolicy: RulePolicy
  confidencePolicy: RulePolicy
  precedence: number
  supersedes: string[]
  scoreGroupTemplate: string
  falsePositiveConsiderations: string[]
  remediationGuidance: string[]
}
