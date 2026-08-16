import { describe, expect, it } from 'vitest'
import { baselinePresentation, formatLearningDuration, isBaselineConfirmationValid, learningElapsedSeconds, requiredConfirmation } from './baseline'
import type { BaselineSummary } from '../../types/baseline'

const summary = (status: BaselineSummary['status'], started = '2026-08-16T12:00:00Z'): BaselineSummary => ({ status, learningStartedAt: started, observationCount: 1, schemaVersion: 1, learningPeriodSeconds: 3_600, lastProcessingDurationMs: 4, entities: { executables: 1, processPatterns: 1, parentChildRelationships: 0, networkDestinations: 0, services: 1, networkConfigurations: 1 } })

describe('behavioral baseline presentation', () => {
  it('keeps every backend state explicit and does not imply a score', () => {
    expect(baselinePresentation('learning').label).toBe('Learning')
    expect(baselinePresentation('ready').detail).toContain('factual')
    expect(baselinePresentation('stale').label).toBe('Stale')
    expect(baselinePresentation('error').tone).toBe('danger')
  })

  it('formats learning duration from UTC timestamps', () => {
    expect(learningElapsedSeconds(summary('learning'), Date.parse('2026-08-16T12:18:30Z'))).toBe(1_110)
    expect(formatLearningDuration(1_110)).toBe('18m')
  })

  it('requires the exact strong confirmation phrase for reset and relearn', () => {
    expect(requiredConfirmation('reset')).toBe('RESET BASELINE')
    expect(isBaselineConfirmationValid('reset', 'reset baseline')).toBe(false)
    expect(isBaselineConfirmationValid('reset', 'RESET BASELINE')).toBe(true)
    expect(isBaselineConfirmationValid('start', 'START NEW BASELINE')).toBe(true)
  })
})
