import type { BaselineState, BaselineSummary } from '../../types/baseline'

export type BaselineActionMode = 'start' | 'reset' | 'complete'

export const requiredConfirmation = (mode: BaselineActionMode) => mode === 'reset' ? 'RESET BASELINE' : mode === 'complete' ? 'COMPLETE BASELINE' : 'START NEW BASELINE'
export const isBaselineConfirmationValid = (mode: BaselineActionMode, value: string) => value === requiredConfirmation(mode)

export const baselinePresentation = (status: BaselineState) => ({
  not_initialized: { key: 'not_initialized', tone: 'neutral' as const },
  learning: { key: 'learning', tone: 'warning' as const },
  ready: { key: 'ready', tone: 'good' as const },
  stale: { key: 'stale', tone: 'warning' as const },
  error: { key: 'error', tone: 'danger' as const },
})[status]

export function learningElapsedSeconds(baseline: BaselineSummary, now = Date.now()) {
  if (!baseline.learningStartedAt) return 0
  return Math.max(0, Math.floor((now - new Date(baseline.learningStartedAt).getTime()) / 1000))
}

export function formatLearningDuration(seconds: number, locale?: string) {
  const number = new Intl.NumberFormat(locale ?? 'en-US')
  if (seconds < 60) return `${number.format(seconds)}s`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${number.format(minutes)}m`
  const hours = Math.floor(minutes / 60)
  const remainingMinutes = minutes % 60
  if (hours < 24) return `${number.format(hours)}h ${number.format(remainingMinutes)}m`
  return `${number.format(Math.floor(hours / 24))}d ${number.format(hours % 24)}h`
}
