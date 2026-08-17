import { createInstance } from 'i18next'
import { describe, expect, it } from 'vitest'
import type { SecurityScore } from '../../types/score'
import ptBR from '../../i18n/locales/pt-BR/score'
import { formatBreakdownValue, scoreAvailability } from './scorePresentation'

const score = (overrides: Partial<SecurityScore> = {}): SecurityScore => ({
  state: 'available', score: 94, label: 'Excellent', formulaVersion: 2,
  activeDetectionCount: 2, coverage: [], breakdown: [], detectionPenalty: 6,
  vulnerabilityPenalty: 0,
  vulnerabilityCoverage: { status: 'not_applicable', basis: 'software_record_proxy', totalSoftware: 0, eligibleSoftware: 0, resolvedEligible: 0, ambiguousEligible: 0, unresolvedEligible: 0, notMappable: 0, pendingEvaluation: 0 },
  productVulnerabilityRisks: [], ...overrides,
})

describe('Security Score presentation', () => {
  it('shows a numeric score only for a valid available state', () => {
    expect(scoreAvailability(score())).toMatchObject({ available: true, value: 94 })
    expect(scoreAvailability(score({ state: 'limited', score: 86, reason: 'Limited coverage — Vulnerability Intelligence is updating' }))).toMatchObject({ available: true, limited: true, value: 86 })
    expect(scoreAvailability(score({ state: 'unavailable', score: undefined, reason: 'Baseline is learning' })).available).toBe(false)
    expect(scoreAvailability(score({ score: 101 })).available).toBe(false)
  })

  it('keeps the breakdown arithmetic explicit', () => {
    expect(formatBreakdownValue(-8)).toBe('-8')
    expect(formatBreakdownValue(3)).toBe('+3')
  })

  it('localizes the score label supplied by the engine without deriving score semantics', async () => {
    const instance = createInstance()
    await instance.init({ lng: 'pt-BR', resources: { 'pt-BR': { score: ptBR } }, defaultNS: 'score' })
    const presentation = scoreAvailability(score(), instance.getFixedT('pt-BR', 'score'))
    expect(presentation).toMatchObject({ available: true, title: 'Excelente', value: 94 })
    expect(presentation.detail).toBe('2 detecções ativas na cobertura atual')
    expect(scoreAvailability(score({ label: 'Good' }), instance.getFixedT('pt-BR', 'score')).title).toBe('Boa')
    expect(scoreAvailability(score({ state: 'limited', reason: 'Limited coverage — Vulnerability Intelligence is updating' }), instance.getFixedT('pt-BR', 'score')).detail).toBe('Cobertura limitada — a Inteligência de Vulnerabilidades está atualizando')
    expect(scoreAvailability(score({ state: 'unavailable', score: undefined, reason: 'Unexpected Rust detail' }), instance.getFixedT('pt-BR', 'score')).detail).toBe('Os dados necessários de baseline, coleta ou detecção estão indisponíveis.')
  })
})
