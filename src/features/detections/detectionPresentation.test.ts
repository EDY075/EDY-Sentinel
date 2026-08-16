import { createInstance } from 'i18next'
import { describe, expect, it } from 'vitest'
import ptBR from '../../i18n/locales/pt-BR/detections'
import { confidenceLabel, detectionStatusLabel, evidenceValue, localizedCollectorSource, localizedDetectionField, localizedEntityType, localizedEvidenceField, localizedEvidenceLabel, localizedEvidenceType, severityLabel, severityTone } from './detectionPresentation'

describe('detection presentation', () => {
  const backendEvidenceFields = [
    'account', 'after', 'afterAddress', 'afterInterface', 'afterMetric', 'architecture',
    'association', 'baselineVersion', 'before', 'beforeAddress', 'beforeInterface',
    'beforeMetric', 'binaryPath', 'child', 'childExecutableKey', 'childPath', 'company',
    'comparison', 'correlationCycle', 'displayName', 'executableKey', 'field',
    'fileModifiedAt', 'fileSize', 'firstObserved', 'firstObservedMissing', 'interfaceType',
    'lastKnownStartupType', 'parent', 'parentExecutableKey', 'parentPath', 'parentProcess',
    'path', 'process', 'protocol', 'remoteIp', 'remotePort', 'runtimePid', 'serviceKey',
    'serviceName', 'signatureStatus', 'signer', 'startupType', 'status', 'user',
  ]

  it('always pairs semantic tones with readable labels', () => {
    expect(severityLabel('informational')).toBe('Informational')
    expect(severityTone('high')).toBe('danger')
    expect(confidenceLabel('medium')).toBe('Medium')
    expect(detectionStatusLabel('acknowledged')).toBe('Acknowledged')
  })

  it('renders evidence without inventing missing values', () => {
    expect(evidenceValue(null)).toBe('Unavailable')
    expect(evidenceValue({ path: 'C:\\test.exe' })).toBe('{"path":"C:\\\\test.exe"}')
  })

  it('maps stable severity, confidence, status and rule identifiers in Portuguese', async () => {
    const instance = createInstance()
    await instance.init({ lng: 'pt-BR', resources: { 'pt-BR': { detections: ptBR } }, defaultNS: 'detections' })
    const t = instance.getFixedT('pt-BR', 'detections')
    expect(severityLabel('medium', t)).toBe('Média')
    expect(confidenceLabel('high', t)).toBe('Alta')
    expect(detectionStatusLabel('investigating', t)).toBe('Em investigação')
    expect(localizedDetectionField('EDY-NET-002', 'title', 'fallback', t)).toContain('Gateway e DNS')
    expect(localizedEvidenceLabel('Unsigned temporary executable', t)).toBe('Executável temporário não assinado')
    expect(localizedEvidenceLabel('service_binary_changed', t)).toBe('Binário do serviço alterado')
    expect(localizedEvidenceType('gateway_configuration', t)).toBe('Configuração do gateway')
    expect(localizedEvidenceField('signatureStatus', t)).toBe('Status da assinatura')
    expect(localizedEvidenceField('parentPath', t)).toBe('Caminho do processo pai')
    expect(localizedEvidenceField('remoteIp', t)).toBe('IP remoto')
    expect(localizedEntityType('process', t)).toBe('Processo')
    expect(localizedCollectorSource('processes', t)).toBe('Processos')
    expect(localizedEvidenceLabel('Future backend label', t)).toBe('Future backend label')
    expect(localizedEntityType('future_entity', t)).toBe('future_entity')
  })

  it('covers every evidence field emitted by the current baseline engine', () => {
    for (const field of backendEvidenceFields) expect(ptBR.evidenceFields).toHaveProperty(field)
  })
})
