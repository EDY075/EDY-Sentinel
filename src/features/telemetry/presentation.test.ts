import { createInstance } from 'i18next'
import { describe, expect, it } from 'vitest'
import connectionsPtBR from '../../i18n/locales/pt-BR/connections'
import processesPtBR from '../../i18n/locales/pt-BR/processes'
import servicesPtBR from '../../i18n/locales/pt-BR/services'
import { domainValueKey, localizedDomainValue } from './presentation'

describe('operational domain-value presentation', () => {
  it('normalizes stable backend values only for translation lookup', () => {
    expect(domainValueKey('Automatic Delayed')).toBe('automatic_delayed')
    expect(domainValueKey('fin_wait_1')).toBe('fin_wait_1')
  })

  it('localizes known values without changing their source representation', async () => {
    const instance = createInstance()
    await instance.init({
      lng: 'pt-BR',
      resources: { 'pt-BR': { processes: processesPtBR, services: servicesPtBR, connections: connectionsPtBR } },
    })

    expect(localizedDomainValue('signed', 'signatureStatus', instance.getFixedT('pt-BR', 'processes'))).toBe('Assinado')
    expect(localizedDomainValue('Automatic Delayed', 'startupType', instance.getFixedT('pt-BR', 'services'))).toBe('Automática (atrasada)')
    expect(localizedDomainValue('syn_sent', 'tcpState', instance.getFixedT('pt-BR', 'connections'))).toBe('SYN enviado')
    expect(localizedDomainValue('future_state', 'tcpState', instance.getFixedT('pt-BR', 'connections'))).toBe('future_state')
  })
})
