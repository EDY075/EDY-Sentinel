import type { DetectionConfidence, DetectionSeverity, DetectionStatus } from '../../types/detection'
import type { TFunction } from 'i18next'

export const titleCase = (value: string) => value.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ')
export const severityLabel = (severity: DetectionSeverity, t?: TFunction<'detections'>) => t?.(`severity.${severity}`, { defaultValue: titleCase(severity) }) ?? titleCase(severity)
export const confidenceLabel = (confidence: DetectionConfidence, t?: TFunction<'detections'>) => t?.(`confidence.${confidence}`, { defaultValue: titleCase(confidence) }) ?? titleCase(confidence)
export const detectionStatusLabel = (status: DetectionStatus, t?: TFunction<'detections'>) => t?.(`status.${status}`, { defaultValue: titleCase(status) }) ?? titleCase(status)
export const severityTone = (severity: DetectionSeverity) => severity === 'critical' || severity === 'high' ? 'danger' as const : severity === 'medium' ? 'warning' as const : severity === 'low' ? 'accent' as const : 'neutral' as const
export const confidenceTone = (confidence: DetectionConfidence) => confidence === 'high' ? 'good' as const : confidence === 'medium' ? 'accent' as const : 'neutral' as const

export function evidenceValue(value: unknown, unavailable = 'Unavailable') {
  if (value == null) return unavailable
  if (typeof value === 'string') return value === 'Unavailable' ? unavailable : value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  return JSON.stringify(value)
}

export type DetectionPresentationField = 'title' | 'summary' | 'whatHappened' | 'whyFlagged' | 'severityReason' | 'confidenceReason'

export function localizedDetectionField(ruleId: string, field: DetectionPresentationField, fallback: string, t: TFunction<'detections'>) {
  return t(`presentations.${ruleId}.${field}`, { defaultValue: fallback })
}

export function localizedDetectionGuidance(ruleId: string, fallback: string[], t: TFunction<'detections'>) {
  return fallback.map((item, index) => t(`presentations.${ruleId}.guidance.${index}`, { defaultValue: item }))
}

const EVIDENCE_LABEL_KEYS: Readonly<Record<string, string>> = {
  'Unsigned temporary executable': 'evidenceLabels.unsignedTemporaryExecutable',
  'Executed process pattern': 'evidenceLabels.executedProcessPattern',
  'New parent-child relationship': 'evidenceLabels.newParentChildRelationship',
  'New associated destination': 'evidenceLabels.newAssociatedDestination',
  'New automatic user-writable-path service': 'evidenceLabels.newAutomaticUserWritableService',
  service_startup_changed: 'evidenceLabels.serviceStartupChanged',
  service_binary_changed: 'evidenceLabels.serviceBinaryChanged',
  service_account_changed: 'evidenceLabels.serviceAccountChanged',
  'Gateway changed': 'evidenceLabels.gatewayChanged',
  'DNS resolvers changed': 'evidenceLabels.dnsResolversChanged',
}

export function localizedEvidenceLabel(label: string, t: TFunction<'detections'>) {
  const key = EVIDENCE_LABEL_KEYS[label]
  return key ? t(key, { defaultValue: label }) : label
}

export function localizedEvidenceType(value: string, t: TFunction<'detections'>) {
  return t(`evidenceTypes.${value}`, { defaultValue: titleCase(value) })
}

export function localizedEvidenceField(value: string, t: TFunction<'detections'>) {
  return t(`evidenceFields.${value}`, { defaultValue: titleCase(value) })
}

export function localizedEntityType(value: string, t: TFunction<'detections'>) {
  return t(`entityTypes.${value}`, { defaultValue: value })
}

export function localizedCollectorSource(value: string, t: TFunction<'detections'>) {
  return t(`collectorSources.${value}`, { defaultValue: value })
}

const RULE_BY_SOURCE_TITLE: Readonly<Record<string, string>> = {
  'New unsigned executable ran from a temporary path': 'EDY-PROC-001',
  'New parent-child relationship launched an unsigned temporary executable': 'EDY-PROC-002',
  'New unsigned temporary executable opened a new outbound destination': 'EDY-NET-001',
  'New automatic service uses a binary from a user-writable path': 'EDY-SVC-001',
  'Multiple persistent service properties changed together': 'EDY-SVC-002',
  'Gateway and DNS changed without a primary-route transition': 'EDY-NET-002',
}

export function localizedDetectionTitle(sourceTitle: string, t: TFunction<'detections'>) {
  const ruleId = RULE_BY_SOURCE_TITLE[sourceTitle]
  return ruleId ? t(`presentations.${ruleId}.title`, { defaultValue: sourceTitle }) : sourceTitle
}
