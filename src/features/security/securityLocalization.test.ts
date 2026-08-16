import { describe, expect, it } from 'vitest'
import detectionsEn from '../../i18n/locales/en/detections'
import eventsEn from '../../i18n/locales/en/events'
import rulesEn from '../../i18n/locales/en/rules'
import scoreEn from '../../i18n/locales/en/score'
import securityEn from '../../i18n/locales/en/security'
import detectionsPtBR from '../../i18n/locales/pt-BR/detections'
import eventsPtBR from '../../i18n/locales/pt-BR/events'
import rulesPtBR from '../../i18n/locales/pt-BR/rules'
import scorePtBR from '../../i18n/locales/pt-BR/score'
import securityPtBR from '../../i18n/locales/pt-BR/security'

function leafKeys(value: unknown, prefix = ''): string[] {
  if (!value || typeof value !== 'object') return [prefix]
  return Object.entries(value).flatMap(([key, entry]) => leafKeys(entry, prefix ? `${prefix}.${key}` : key)).sort()
}

describe('security workspace locale catalogs', () => {
  it.each([
    ['events', eventsEn, eventsPtBR],
    ['detections', detectionsEn, detectionsPtBR],
    ['rules', rulesEn, rulesPtBR],
    ['score', scoreEn, scorePtBR],
    ['security', securityEn, securityPtBR],
  ])('keeps en and pt-BR %s keys in parity', (_namespace, en, ptBR) => {
    expect(leafKeys(ptBR)).toEqual(leafKeys(en))
  })

  it('keys all built-in rule presentation by immutable Rule ID', () => {
    const expected = ['EDY-NET-001', 'EDY-NET-002', 'EDY-PROC-001', 'EDY-PROC-002', 'EDY-SVC-001', 'EDY-SVC-002']
    expect(Object.keys(rulesPtBR.definitions).sort()).toEqual(expected)
    expect(Object.keys(detectionsPtBR.presentations).sort()).toEqual(expected)
  })
})
