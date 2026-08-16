import type { BaselineState, BaselineSummary } from '../../types/baseline'

export type BaselineActionMode = 'start' | 'reset' | 'complete'

export const requiredConfirmation = (mode: BaselineActionMode) => mode === 'reset' ? 'RESET BASELINE' : mode === 'complete' ? 'COMPLETE BASELINE' : 'START NEW BASELINE'
export const isBaselineConfirmationValid = (mode: BaselineActionMode, value: string) => value === requiredConfirmation(mode)

export const baselinePresentation = (status: BaselineState) => ({
  not_initialized: { label: 'Not initialized', tone: 'neutral' as const, detail: 'No behavior has been learned yet.' },
  learning: { label: 'Learning', tone: 'warning' as const, detail: 'Real observations are being added without generating new-behavior events.' },
  ready: { label: 'Ready', tone: 'good' as const, detail: 'New factual differences can create deduplicated security events.' },
  stale: { label: 'Stale', tone: 'warning' as const, detail: 'The baseline has not received a recent successful observation.' },
  error: { label: 'Error', tone: 'danger' as const, detail: 'The baseline could not be updated. Existing telemetry remains available.' },
})[status]

export function learningElapsedSeconds(baseline: BaselineSummary, now = Date.now()) {
  if (!baseline.learningStartedAt) return 0
  return Math.max(0, Math.floor((now - new Date(baseline.learningStartedAt).getTime()) / 1000))
}

export function formatLearningDuration(seconds: number) {
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  if (hours < 24) return `${hours}h ${remainingMinutes}m`
  return `${Math.floor(hours / 24)}d ${hours % 24}h`
}
