import { describe, expect, it } from 'vitest'
import type { RuleDefinition } from './rules'

describe('RuleDefinition contract', () => {
  it('keeps rule and policy versions through serialization without executing a rule', () => {
    const definition: RuleDefinition = {
      ruleId: 'EDY-CONTRACT-001',
      version: 1,
      name: 'Contract fixture',
      description: 'Typed infrastructure only',
      category: 'contract_test',
      enabled: false,
      conditions: [{ factType: 'factual_event', field: 'eventType', operator: 'equals', expected: 'fixture' }],
      requiredEvidence: [{ factType: 'factual_event', minimumCount: 2, correlationFields: ['entityKey'] }],
      severityPolicy: { policyId: 'corroborated-facts-only', version: 1, description: 'Novelty alone is insufficient', parameters: {} },
      confidencePolicy: { policyId: 'evidence-correlation-quality', version: 1, description: 'Evidence quality, not malware probability', parameters: {} },
      remediationGuidance: ['Review factual evidence'],
    }

    const restored = JSON.parse(JSON.stringify(definition)) as RuleDefinition
    expect(restored.ruleId).toBe('EDY-CONTRACT-001')
    expect(restored.version).toBe(1)
    expect(restored.severityPolicy.version).toBe(1)
    expect(restored.requiredEvidence[0].minimumCount).toBe(2)
    expect(restored.enabled).toBe(false)
  })
})
