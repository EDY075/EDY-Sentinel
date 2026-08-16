import type { CollectorHealth, CollectorKey, CollectorState, LiveTelemetrySnapshot } from '../../types/telemetry'
import type { SystemOverview } from '../../types/system'

const labels: Record<CollectorKey, string> = { system: 'System', processes: 'Processes', network: 'Network', services: 'Services' }

export function deriveCollectorHealth(loading: boolean, overview: SystemOverview | null, snapshot: LiveTelemetrySnapshot | null, failures: { system?: string; live?: string } = {}): CollectorHealth[] {
  return (Object.keys(labels) as CollectorKey[]).map((key) => {
    if (key === 'system') {
      const issue = failures.system ?? overview?.issues[0]?.message
      const state: CollectorState = loading ? 'loading' : failures.system ? (overview ? 'partial' : 'error') : issue ? 'partial' : overview ? 'active' : 'error'
      return { key, label: labels[key], state, issue, collectedAt: overview?.collectedAt }
    }
    const ids = key === 'network' ? ['network', 'connections'] : [key]
    const collector = snapshot?.collectors.find(({ id }) => ids.includes(id))
    const issue = failures.live ?? snapshot?.issues.find(({ component }) => ids.some((id) => component === id || component.startsWith(`${id}-`)))?.message
    const state: CollectorState = loading ? 'loading' : failures.live ? (snapshot ? 'partial' : 'error') : collector?.status ?? (snapshot ? 'partial' : 'error')
    return { key, label: labels[key], state, issue: issue ?? collector?.detail, collectedAt: collector?.collectedAt }
  })
}
