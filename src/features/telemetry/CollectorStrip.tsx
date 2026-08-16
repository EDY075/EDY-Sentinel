import { CircleAlert, Pause, Play, RefreshCw } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Tooltip } from '../../components/ui/primitives'
import type { CollectorHealth } from '../../types/telemetry'

export function CollectorStrip({ health, live, refreshing, onLiveChange, onRefresh }: { health: CollectorHealth[]; live: boolean; refreshing: boolean; onLiveChange: (live: boolean) => void; onRefresh: () => void }) {
  const { t, i18n } = useTranslation('telemetry')
  const number = new Intl.NumberFormat(i18n.resolvedLanguage ?? i18n.language)
  const collectorName = (collector: CollectorHealth) => t(`collectors.names.${collector.key}`)
  const stateLabel = (state: CollectorHealth['state']) => t(`collectors.states.${state}`)
  const collectorSummary = (collector: CollectorHealth) => {
    const observed = t('collectors.observed', { count: collector.observationCount, formattedCount: number.format(collector.observationCount) })
    const restricted = collector.restrictedCount ? ` · ${t('collectors.restricted', { count: collector.restrictedCount, formattedCount: number.format(collector.restrictedCount) })}` : ''
    return t('collectors.summary', {
      label: collectorName(collector),
      state: stateLabel(collector.state),
      coverage: `${observed}${restricted}`,
      timing: collector.durationMs === undefined ? '' : t('collectors.timing', { duration: number.format(collector.durationMs) }),
      issue: collector.issue ? t('collectors.issue') : '',
    })
  }
  return (
    <div className="collector-strip" aria-label={t('collectors.ariaLabel')}>
      <div className="collector-strip__items"><span className="collector-strip__label">{t('collectors.label')}</span>{health.map((collector) => <Tooltip key={collector.key} label={collectorSummary(collector)}><span className="collector-health" data-state={collector.state}><i />{collectorName(collector)}<small>{stateLabel(collector.state)}</small>{collector.restrictedCount > 0 && <em>{t('collectors.restrictedBadge', { count: collector.restrictedCount, formattedCount: number.format(collector.restrictedCount) })}</em>}</span></Tooltip>)}</div>
      <div className="live-controls">
        {health.some(({ state }) => state === 'degraded' || state === 'failed') && <Tooltip label={t('collectors.warning')}><CircleAlert size={15} className="collector-warning" /></Tooltip>}
        <button type="button" className="live-toggle" data-live={live} aria-pressed={live} onClick={() => onLiveChange(!live)}>{live ? <><Pause size={13} /> {t('collectors.live')}</> : <><Play size={13} /> {t('collectors.paused')}</>}</button>
        <button type="button" className="icon-button icon-button--small" aria-label={t('collectors.refresh')} disabled={refreshing} onClick={onRefresh}><RefreshCw size={14} className={refreshing ? 'spin' : ''} /></button>
      </div>
    </div>
  )
}
