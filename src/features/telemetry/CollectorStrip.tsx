import { CircleAlert, Pause, Play, RefreshCw } from 'lucide-react'
import { Tooltip } from '../../components/ui/primitives'
import type { CollectorHealth } from '../../types/telemetry'

const stateLabel = (state: CollectorHealth['state']) => state.charAt(0).toUpperCase() + state.slice(1)

function collectorSummary(collector: CollectorHealth) {
  const coverage = `${collector.observationCount} observed${collector.restrictedCount ? ` · ${collector.restrictedCount} with restricted metadata` : ''}`
  const timing = collector.durationMs === undefined ? '' : ` · ${collector.durationMs} ms`
  return `${collector.label}: ${stateLabel(collector.state)}. ${coverage}${timing}. ${collector.issue ?? ''}`.trim()
}

export function CollectorStrip({ health, live, refreshing, onLiveChange, onRefresh }: { health: CollectorHealth[]; live: boolean; refreshing: boolean; onLiveChange: (live: boolean) => void; onRefresh: () => void }) {
  return (
    <div className="collector-strip" aria-label="Collector health">
      <div className="collector-strip__items"><span className="collector-strip__label">Collectors</span>{health.map((collector) => <Tooltip key={collector.key} label={collectorSummary(collector)}><span className="collector-health" data-state={collector.state}><i />{collector.label}<small>{stateLabel(collector.state)}</small>{collector.restrictedCount > 0 && <em>{collector.restrictedCount} restricted</em>}</span></Tooltip>)}</div>
      <div className="live-controls">
        {health.some(({ state }) => state === 'degraded' || state === 'failed') && <Tooltip label="One or more collectors reported a factual collection failure"><CircleAlert size={15} className="collector-warning" /></Tooltip>}
        <button type="button" className="live-toggle" data-live={live} aria-pressed={live} onClick={() => onLiveChange(!live)}>{live ? <><Pause size={13} /> Live</> : <><Play size={13} /> Paused</>}</button>
        <button type="button" className="icon-button icon-button--small" aria-label="Refresh all telemetry" disabled={refreshing} onClick={onRefresh}><RefreshCw size={14} className={refreshing ? 'spin' : ''} /></button>
      </div>
    </div>
  )
}
