import { createInstance } from 'i18next'
import { describe, expect, it } from 'vitest'
import baselineEn from './locales/en/baseline'
import connectionsEn from './locales/en/connections'
import overviewEn from './locales/en/overview'
import processesEn from './locales/en/processes'
import servicesEn from './locales/en/services'
import telemetryEn from './locales/en/telemetry'
import baselinePtBR from './locales/pt-BR/baseline'
import connectionsPtBR from './locales/pt-BR/connections'
import overviewPtBR from './locales/pt-BR/overview'
import processesPtBR from './locales/pt-BR/processes'
import servicesPtBR from './locales/pt-BR/services'
import telemetryPtBR from './locales/pt-BR/telemetry'
import { flattenResourceKeys } from './resources'

const en = { baseline: baselineEn, connections: connectionsEn, overview: overviewEn, processes: processesEn, services: servicesEn, telemetry: telemetryEn }
const ptBR = { baseline: baselinePtBR, connections: connectionsPtBR, overview: overviewPtBR, processes: processesPtBR, services: servicesPtBR, telemetry: telemetryPtBR }

describe('operational translation resources', () => {
  it('keeps English and Brazilian Portuguese keys aligned by domain', () => {
    for (const namespace of Object.keys(en) as Array<keyof typeof en>) {
      expect(flattenResourceKeys(ptBR[namespace])).toEqual(flattenResourceKeys(en[namespace]))
    }
  })

  it('uses real locale plural rules for operational counts', async () => {
    const instance = createInstance()
    await instance.init({ lng: 'pt-BR', fallbackLng: 'en', resources: { en, 'pt-BR': ptBR } })
    expect(instance.t('panel.executables', { ns: 'baseline', count: 1, formattedCount: '1' })).toBe('1 executável')
    expect(instance.t('panel.executables', { ns: 'baseline', count: 2, formattedCount: '2' })).toBe('2 executáveis')
    expect(instance.t('collectors.observed', { ns: 'telemetry', count: 2, formattedCount: '2' })).toBe('2 observados')
  })

  it('uses the approved Brazilian Portuguese primary-route terminology', () => {
    expect(overviewPtBR.network.primaryRoute).toBe('Rota Principal')
  })
})
