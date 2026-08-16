import { CircleAlert, Pause, Play, RefreshCw } from 'lucide-react'
import { Tooltip } from '../../components/ui/primitives'
import type { CollectorHealth } from '../../types/telemetry'

export function CollectorStrip({ health, live, refreshing, onLiveChange, onRefresh }: { health: CollectorHealth[]; live: boolean; refreshing: boolean; onLiveChange: (live: boolean) => void; onRefresh: () => void }) {
  return (
    <div className="collector-strip" aria-label="Collector health">
      <div className="collector-strip__items"><span className="collector-strip__label">Collectors</span>{health.map((collector) => <Tooltip key={collector.key} label={collector.issue ?? `${collector.label} collector is ${collector.state}`}><span className="collector-health" data-state={collector.state}><i />{collector.label}<small>{collector.state}</small></span></Tooltip>)}</div>
      <div className="live-controls">
        {health.some(({ state }) => state === 'partial' || state === 'error') && <Tooltip label="One or more collectors returned partial telemetry"><CircleAlert size={15} className="collector-warning" /></Tooltip>}
        <button type="button" className="live-toggle" data-live={live} aria-pressed={live} onClick={() => onLiveChange(!live)}>{live ? <><Pause size={13} /> Live</> : <><Play size={13} /> Paused</>}</button>
        <button type="button" className="icon-button icon-button--small" aria-label="Refresh all telemetry" disabled={refreshing} onClick={onRefresh}><RefreshCw size={14} className={refreshing ? 'spin' : ''} /></button>
      </div>
    </div>
  )
}
