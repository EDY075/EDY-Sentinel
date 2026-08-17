import type { SecurityScore } from '../../types/score'
import type { TFunction } from 'i18next'
import { formatNumber } from '../../i18n'

const SCORE_REASON_KEYS: Readonly<Record<string, string>> = {
  'Score unavailable — no behavioral baseline exists': 'reasons.noBaseline',
  'Score unavailable — baseline still learning': 'reasons.learning',
  'Score unavailable — baseline coverage is stale': 'reasons.stale',
  'Score unavailable — baseline is in Error': 'reasons.error',
  'Score unavailable — baseline is not ready': 'reasons.notReady',
  'Score unavailable — no detection rules are enabled': 'reasons.noRules',
  'Score unavailable — required collector coverage is incomplete': 'reasons.incompleteCoverage',
  'Limited coverage — Vulnerability Intelligence is updating': 'reasons.vulnerabilityUpdating',
}

function translatedReason(reason: string | undefined, fallback: string, t?: TFunction<'score'>) {
  if (!reason) return fallback
  const key = SCORE_REASON_KEYS[reason]
  if (t) return key ? t(key, { defaultValue: fallback }) : fallback
  return reason
}

const SCORE_LABEL_KEYS: Readonly<Record<string, string>> = {
  Excellent: 'labels.excellent',
  Good: 'labels.good',
  Attention: 'labels.attention',
  'Elevated Risk': 'labels.elevatedRisk',
}

function scoreLabel(label: string | undefined, t?: TFunction<'score'>) {
  const fallback = label ?? (t?.('labels.observedPosture') ?? 'Observed posture')
  const key = label ? SCORE_LABEL_KEYS[label] : undefined
  return key && t ? t(key, { defaultValue: fallback }) : fallback
}

export function scoreAvailability(score: SecurityScore | null, t?: TFunction<'score'>) {
  if (!score) return { available: false, title: t?.('availability.noScoreTitle') ?? 'Security Score unavailable', detail: t?.('availability.noScoreDetail') ?? 'Security analysis has not returned a score state.' } as const
  if (score.state === 'limited') {
    const fallback = t?.('availability.limitedDetail') ?? 'Minimum score coverage is not currently available.'
    if (score.score == null || score.score < 0 || score.score > 100) return { available: false, title: t?.('state.limited') ?? 'Limited coverage', detail: translatedReason(score.reason, fallback, t) } as const
    return { available: true, limited: true, title: t?.('state.limitedAnalysis') ?? 'Analysis has limited coverage', scoreLabel: scoreLabel(score.label, t), detail: translatedReason(score.reason, fallback, t), value: score.score } as const
  }
  if (score.state === 'unavailable') {
    const fallback = t?.('availability.unavailableDetail') ?? 'Required baseline, collection or detection data is unavailable.'
    return { available: false, title: t?.('state.unavailable') ?? 'Score unavailable', detail: translatedReason(score.reason, fallback, t) } as const
  }
  if (score.score == null || score.score < 0 || score.score > 100) return { available: false, title: t?.('availability.invalidTitle') ?? 'Security Score unavailable', detail: t?.('availability.invalidDetail') ?? 'The engine returned an invalid score value.' } as const
  const detail = t?.('availability.activeDetections', { count: score.activeDetectionCount, formattedCount: formatNumber(score.activeDetectionCount) }) ?? `${score.activeDetectionCount} active detection${score.activeDetectionCount === 1 ? '' : 's'} under current coverage`
  return { available: true, limited: false, title: scoreLabel(score.label, t), scoreLabel: scoreLabel(score.label, t), detail, value: score.score } as const
}

export const formatBreakdownValue = (value: number, formatter: (input: number) => string = String) => value > 0 ? `+${formatter(value)}` : formatter(value)
