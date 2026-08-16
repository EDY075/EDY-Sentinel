import { describe, expect, it } from 'vitest'
import type { SecurityScore } from '../../types/score'
import { formatBreakdownValue, scoreAvailability } from './scorePresentation'

const score = (overrides: Partial<SecurityScore> = {}): SecurityScore => ({ state: 'available', score: 94, label: 'Good', formulaVersion: 1, activeDetectionCount: 2, coverage: [], breakdown: [], ...overrides })

describe('Security Score presentation', () => {
  it('shows a numeric score only for a valid available state', () => {
    expect(scoreAvailability(score())).toMatchObject({ available: true, value: 94 })
    expect(scoreAvailability(score({ state: 'unavailable', score: undefined, reason: 'Baseline is learning' })).available).toBe(false)
    expect(scoreAvailability(score({ score: 101 })).available).toBe(false)
  })

  it('keeps the breakdown arithmetic explicit', () => {
    expect(formatBreakdownValue(-8)).toBe('-8')
    expect(formatBreakdownValue(3)).toBe('+3')
  })
})
