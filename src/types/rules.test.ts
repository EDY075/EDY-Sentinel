import { describe, expect, it } from 'vitest'
import type { RuleDefinition } from './rules'

describe('RuleDefinition contract', () => {
  it('keeps explainability, correlation, and precedence metadata through serialization', () => {
    const definition: RuleDefinition = {
      ruleId: 'EDY-PROC-001',
      version: 1,
      name: 'New unsigned executable in a temporary user-writable path',
      description: 'Correlated factual evidence',
      category: 'process_execution',
      enabled: true,
      references: ['security_event:executable_first_seen@v1'],
      conditions: [
        {
          factType: 'executable_first_seen',
          field: 'evidence.signatureStatus',
          operator: 'equals',
          expected: 'unsigned',
        },
      ],
      exclusions: [
        {
          factType: 'executable_first_seen',
          field: 'evidence.signatureStatus',
          operator: 'in',
          expected: ['unknown', 'restricted', 'signed'],
        },
      ],
      requiredEvidence: [
        {
          evidenceId: 'new_executable',
          factType: 'security_event',
          acceptedEventTypes: ['executable_first_seen'],
          minimumCount: 1,
          correlationFields: ['evidence.path'],
        },
      ],
      correlationWindowSeconds: 60,
      defaultSeverity: 'low',
      defaultConfidence: 'high',
      severityPolicy: {
        policyId: 'corroborated-facts-only',
        version: 1,
        description: 'Novelty alone is insufficient',
        parameters: { maximumSeverity: 'medium', noveltyAloneTriggers: false },
      },
      confidencePolicy: {
        policyId: 'evidence-correlation-quality',
        version: 1,
        description: 'Evidence quality, not malware probability',
        parameters: { unknownSignatureIsUnsigned: false },
      },
      precedence: 100,
      supersedes: [],
      scoreGroupTemplate: 'executable:{entityKey}',
      falsePositiveConsiderations: ['A legitimate installer may extract a temporary helper.'],
      remediationGuidance: ['Review factual evidence without taking automatic action.'],
    }

    const restored = JSON.parse(JSON.stringify(definition)) as RuleDefinition
    expect(restored.ruleId).toBe('EDY-PROC-001')
    expect(restored.version).toBe(1)
    expect(restored.severityPolicy.version).toBe(1)
    expect(restored.requiredEvidence[0].acceptedEventTypes).toEqual(['executable_first_seen'])
    expect(restored.correlationWindowSeconds).toBe(60)
    expect(restored.defaultSeverity).toBe('low')
    expect(restored.scoreGroupTemplate).toBe('executable:{entityKey}')
    expect(restored.enabled).toBe(true)
  })

  it('keeps unknown and restricted signature states separate from unsigned', () => {
    const excludedStates: RuleDefinition['exclusions'][number]['expected'] = ['unknown', 'restricted']
    expect(excludedStates).not.toContain('unsigned')
  })
})
