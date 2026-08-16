export type RuleOperator = 'equals' | 'not_equals' | 'exists' | 'in' | 'at_least'

export interface RuleCondition {
  factType: string
  field: string
  operator: RuleOperator
  expected: unknown
}

export interface EvidenceRequirement {
  factType: string
  minimumCount: number
  correlationFields: string[]
}

export interface RulePolicy {
  policyId: string
  version: number
  description: string
  parameters: Record<string, unknown>
}

/** Contract only. Sprint 2 hardening does not evaluate or register rules. */
export interface RuleDefinition {
  ruleId: string
  version: number
  name: string
  description: string
  category: string
  enabled: boolean
  conditions: RuleCondition[]
  requiredEvidence: EvidenceRequirement[]
  severityPolicy: RulePolicy
  confidencePolicy: RulePolicy
  remediationGuidance: string[]
}
