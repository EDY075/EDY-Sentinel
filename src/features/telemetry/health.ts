import type { CollectorHealth, CollectorKey, CollectorState, LiveTelemetrySnapshot } from '../../types/telemetry'
import type { SystemOverview } from '../../types/system'

const labels: Record<CollectorKey, string> = { system: 'System', processes: 'Processes', network: 'Network', services: 'Services' }

export function deriveCollectorHealth(loading: boolean, overview: SystemOverview | null, snapshot: LiveTelemetrySnapshot | null, failures: { system?: string; live?: string } = {}): CollectorHealth[] {
  return (Object.keys(labels) as CollectorKey[]).map((key) => {
    if (key === 'system') {
      const collector = overview?.collector
      const state: CollectorState = loading ? 'loading' : failures.system ? 'failed' : collector?.status ?? 'failed'
      return {
        key,
        label: labels[key],
        state,
        issue: failures.system ?? collector?.errorMessage ?? collector?.detail,
        lastSuccess: collector?.lastSuccess,
        lastAttempt: collector?.lastAttempt,
        durationMs: collector?.durationMs,
        observationCount: collector?.observationCount ?? 0,
        restrictedCount: collector?.restrictedCount ?? 0,
      }
    }
    const ids = key === 'network' ? ['network', 'connections'] : [key]
    const collector = snapshot?.collectors.find(({ id }) => ids.includes(id))
    const issue = failures.live ?? collector?.errorMessage ?? snapshot?.issues.find(({ component }) => ids.some((id) => component === id || component.startsWith(`${id}-`)))?.message
    const state: CollectorState = loading ? 'loading' : failures.live ? 'failed' : collector?.status ?? 'failed'
    return {
      key,
      label: labels[key],
      state,
      issue: issue ?? collector?.detail,
      lastSuccess: collector?.lastSuccess,
      lastAttempt: collector?.lastAttempt,
      durationMs: collector?.durationMs,
      observationCount: collector?.observationCount ?? 0,
      restrictedCount: collector?.restrictedCount ?? 0,
    }
  })
}

export function applyPausedState(health: CollectorHealth[], live: boolean): CollectorHealth[] {
  return live ? health : health.map((collector) => ({ ...collector, state: 'paused' }))
}
