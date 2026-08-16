import type { DetectionConfidence, DetectionSeverity, DetectionStatus } from '../../types/detection'

export const titleCase = (value: string) => value.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ')
export const severityLabel = (severity: DetectionSeverity) => titleCase(severity)
export const confidenceLabel = (confidence: DetectionConfidence) => titleCase(confidence)
export const detectionStatusLabel = (status: DetectionStatus) => titleCase(status)
export const severityTone = (severity: DetectionSeverity) => severity === 'critical' || severity === 'high' ? 'danger' as const : severity === 'medium' ? 'warning' as const : severity === 'low' ? 'accent' as const : 'neutral' as const
export const confidenceTone = (confidence: DetectionConfidence) => confidence === 'high' ? 'good' as const : confidence === 'medium' ? 'accent' as const : 'neutral' as const

export function evidenceValue(value: unknown) {
  if (value == null) return 'Unavailable'
  if (typeof value === 'string') return value
  if (typeof value === 'number' || typeof value === 'boolean') return String(value)
  return JSON.stringify(value)
}
