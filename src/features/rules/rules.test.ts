import { describe, expect, it } from 'vitest'
import type { RuleDefinition } from '../../types/rules'

const rule = { ruleId: 'EDY-PROC-001', version: 1, name: 'Risk-context executable', description: 'Requires corroborating evidence.', category: 'process', enabled: true, defaultSeverity: 'medium', requiredEvidence: [{}, {}, {}] } as RuleDefinition

describe('rule summary contract', () => {
  it('keeps version and enabled state explicit', () => {
    expect(rule.ruleId).toBe('EDY-PROC-001')
    expect(rule.version).toBe(1)
    expect(rule.enabled).toBe(true)
    expect(rule.requiredEvidence.length).toBeGreaterThan(1)
  })
})
