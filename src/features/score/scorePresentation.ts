import type { SecurityScore } from '../../types/score'

export function scoreAvailability(score: SecurityScore | null) {
  if (!score) return { available: false, title: 'Security Score unavailable', detail: 'Security analysis has not returned a score state.' } as const
  if (score.state === 'limited') return { available: false, title: 'Limited coverage', detail: score.reason ?? 'Minimum score coverage is not currently available.' } as const
  if (score.state === 'unavailable') return { available: false, title: 'Score unavailable', detail: score.reason ?? 'Required baseline, collection or detection data is unavailable.' } as const
  if (score.score == null || score.score < 0 || score.score > 100) return { available: false, title: 'Security Score unavailable', detail: 'The engine returned an invalid score value.' } as const
  return { available: true, title: score.label ?? 'Observed posture', detail: `${score.activeDetectionCount} active detection${score.activeDetectionCount === 1 ? '' : 's'} under current coverage`, value: score.score } as const
}

export const formatBreakdownValue = (value: number) => value > 0 ? `+${value}` : String(value)
