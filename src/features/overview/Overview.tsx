import { AlertTriangle, Box, Cpu, Gauge, HardDrive, MemoryStick, MonitorCog, Network, RefreshCw, Router, Server, Wifi } from 'lucide-react'
import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Badge, EmptyState, Skeleton, StatusDot } from '../../components/ui/primitives'
import type { DatabaseStatus, SystemOverview } from '../../types/system'
import type { BaselineSummary } from '../../types/baseline'
import type { SecurityScore } from '../../types/score'
import { BaselinePanel } from '../baseline/BaselinePanel'
import { SecurityScorePanel } from '../score/SecurityScorePanel'
import type { BaselineActionMode } from '../baseline/baseline'
import { formatBytes, formatUptime, percent } from './format'
import { summarizePrimaryRoute } from './network'

interface OverviewProps {
  data: SystemOverview | null
  database: DatabaseStatus | null
  loading: boolean
  error: string | null
  onRefresh: () => void
  baseline: BaselineSummary | null
  onBaselineAction: (mode: BaselineActionMode) => void
  onOpenEvents: () => void
  securityScore: SecurityScore | null
  onOpenScore: () => void
  collectorStrip: ReactNode
}

function MetricCard({ icon, label, value, detail, progress }: { icon: React.ReactNode; label: string; value: string; detail: string; progress?: number }) {
  const { t } = useTranslation('overview')
  return (
    <article className="metric-card">
      <div className="metric-card__head"><span className="metric-icon">{icon}</span><span>{label}</span></div>
      <strong>{value}</strong>
      <p>{detail}</p>
      {progress !== undefined && <div className="progress" aria-label={t('progress.used', { value: progress })}><span style={{ width: `${progress}%` }} /></div>}
    </article>
  )
}

function LoadingOverview() {
  const { t } = useTranslation('overview')
  return (
    <div className="overview-loading" aria-label={t('loading.ariaLabel')}>
      <div className="hero-card"><Skeleton className="skeleton--title" /><Skeleton className="skeleton--line" /></div>
      <div className="metrics-grid">{Array.from({ length: 4 }, (_, index) => <article className="metric-card" key={index}><Skeleton className="skeleton--icon" /><Skeleton className="skeleton--value" /><Skeleton className="skeleton--line" /></article>)}</div>
      <div className="details-grid"><div className="panel"><Skeleton className="skeleton--title" /><Skeleton className="skeleton--block" /></div><div className="panel"><Skeleton className="skeleton--title" /><Skeleton className="skeleton--block" /></div></div>
    </div>
  )
}

export function Overview({ data, database, loading, error, onRefresh, baseline, onBaselineAction, onOpenEvents, securityScore, onOpenScore, collectorStrip }: OverviewProps) {
  const { t, i18n } = useTranslation('overview')
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const localizedValue = (value: string) => value === 'Unavailable' ? t('system.unavailable') : value
  if (loading) return <LoadingOverview />
  if (error) {
    return (
      <div className="error-state" role="alert">
        <span><AlertTriangle size={22} /></span>
        <div><strong>{t('error.title')}</strong><p>{t('error.description')}</p></div>
        <button type="button" className="button" onClick={onRefresh}><RefreshCw size={16} /> {t('error.retry')}</button>
      </div>
    )
  }
  if (!data) return <EmptyState title={t('empty.title')} description={t('empty.description')} />

  const memoryUsed = percent(data.memory.usedBytes, data.memory.totalBytes)
  const diskTotal = data.disks.reduce((sum, disk) => sum + disk.totalBytes, 0)
  const diskUsed = data.disks.reduce((sum, disk) => sum + (disk.totalBytes - disk.availableBytes), 0)
  const osDetail = [data.operatingSystem.edition, data.operatingSystem.displayVersion, data.operatingSystem.build && t('system.build', { value: data.operatingSystem.build })].filter(Boolean).join(' · ')
  const primaryRoute = summarizePrimaryRoute(data.network, { primaryRoute: t('network.primaryRoute'), unavailable: t('network.unavailable'), adapterTypeUnavailable: t('network.adapterTypeUnavailable') })

  return (
    <div className="overview">
      <div className="overview-command-deck">
        <section className="hero-card">
          <div className="hero-card__identity">
            <div className="eyebrow"><StatusDot status={data.issues.length ? 'partial' : 'online'} /> {t('hero.eyebrow')}</div>
            <h1>{localizedValue(data.host.hostname)}</h1>
            <p>{t('hero.signedInAs', { os: localizedValue(data.operatingSystem.name), architecture: data.host.architecture, username: localizedValue(data.host.username) })}</p>
            <div className="hero-card__facts"><span>{data.operatingSystem.displayVersion || data.operatingSystem.edition || t('system.editionUnavailable')}</span><span>{t('hero.logicalProcessors', { count: data.cpu.logicalCores, formattedCount: number.format(data.cpu.logicalCores) })}</span><span>{t('hero.memoryInstalled', { value: formatBytes(data.memory.totalBytes, locale, t('system.unavailable')) })}</span></div>
          </div>
          <div className="hero-card__meta">
            <Badge tone={data.issues.length ? 'warning' : 'good'}>{data.issues.length ? t('hero.incomplete') : t('hero.current')}</Badge>
            <span>{t('hero.updated', { time: new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(new Date(data.collectedAt)) })}</span>
          </div>
        </section>
        <SecurityScorePanel score={securityScore} baselineStatus={baseline?.status ?? 'not_initialized'} onOpen={onOpenScore} />
      </div>

      {collectorStrip}

      <div className="metrics-grid">
        <MetricCard icon={<Cpu size={19} />} label={t('metrics.processor')} value={localizedValue(data.cpu.model)} detail={t('metrics.physicalLogical', { physical: data.cpu.physicalCores == null ? '—' : number.format(data.cpu.physicalCores), logical: number.format(data.cpu.logicalCores) })} />
        <MetricCard icon={<MemoryStick size={19} />} label={t('metrics.memory')} value={t('metrics.used', { value: formatBytes(data.memory.usedBytes, locale, t('system.unavailable')) })} detail={t('metrics.installed', { value: formatBytes(data.memory.totalBytes, locale, t('system.unavailable')) })} progress={memoryUsed} />
        <MetricCard icon={<HardDrive size={19} />} label={t('metrics.storage')} value={t('metrics.used', { value: formatBytes(diskUsed, locale, t('system.unavailable')) })} detail={t('metrics.acrossVolumes', { value: formatBytes(diskTotal, locale, t('system.unavailable')), count: data.disks.length, formattedCount: number.format(data.disks.length) })} progress={percent(diskUsed, diskTotal)} />
        <MetricCard icon={<Gauge size={19} />} label={t('metrics.uptime')} value={formatUptime(data.host.uptimeSeconds, locale)} detail={t('metrics.sinceBoot')} />
      </div>

      <div className="details-grid">
        <section className="panel">
          <header className="panel__header"><div><MonitorCog size={18} /><span><strong>{t('system.title')}</strong><small>{t('system.subtitle')}</small></span></div><Badge>{t('system.realData')}</Badge></header>
          <dl className="detail-list">
            <div><dt><Server size={16} /> {t('system.windows')}</dt><dd>{data.operatingSystem.name}<small>{osDetail || t('system.editionUnavailable')}</small></dd></div>
            <div><dt><Cpu size={16} /> {t('system.cpu')}</dt><dd>{data.cpu.model}<small>{data.cpu.frequencyMhz ? t('system.frequency', { value: number.format(data.cpu.frequencyMhz) }) : t('system.frequencyUnavailable')}</small></dd></div>
            <div><dt><Box size={16} /> {t('system.gpu')}</dt><dd>{data.gpus[0]?.name ?? t('system.unavailable')}<small>{data.gpus.length > 1 ? t('system.adaptersDetected', { count: data.gpus.length, formattedCount: number.format(data.gpus.length) }) : data.gpus[0]?.driverVersion ? t('system.driver', { version: data.gpus[0].driverVersion }) : t('system.noAdditionalDetails')}</small></dd></div>
          </dl>
        </section>

        <section className="panel">
          <header className="panel__header"><div><Network size={18} /><span><strong>{t('network.title')}</strong><small>{t('network.subtitle')}</small></span></div><Badge tone={data.network.primaryIpv4 ? 'good' : 'warning'}>{data.network.primaryIpv4 ? t('network.primaryRoute') : t('network.unavailable')}</Badge></header>
          <dl className="detail-list">
            <div><dt><Network size={16} /> {primaryRoute.label}</dt><dd>{primaryRoute.name}<small>{primaryRoute.type}{data.network.primaryRouteMetric != null ? ` · ${t('network.metric', { value: number.format(data.network.primaryRouteMetric) })}` : ''}</small></dd></div>
            <div><dt><Wifi size={16} /> {t('network.localIp')}</dt><dd>{primaryRoute.localIp}<small>{t('network.additionalInterfaces', { count: primaryRoute.additionalInterfaces, formattedCount: number.format(primaryRoute.additionalInterfaces) })}</small></dd></div>
            <div><dt><Router size={16} /> {t('network.gateway')}</dt><dd>{primaryRoute.gateway}<small>{data.network.dnsServers[0] ? t('network.dns', { value: data.network.dnsServers[0] }) : t('network.dnsUnavailable')}</small></dd></div>
          </dl>
        </section>
      </div>

      <div className="details-grid details-grid--lower">
        <section className="panel">
          <header className="panel__header"><div><HardDrive size={18} /><span><strong>{t('volumes.title')}</strong><small>{t('volumes.subtitle')}</small></span></div><span className="panel-count">{number.format(data.disks.length)}</span></header>
          <div className="volume-list">
            {data.disks.map((disk) => {
              const used = disk.totalBytes - disk.availableBytes
              return <div className="volume-row" key={`${disk.mountPoint}-${disk.name}`}><span className="volume-icon"><HardDrive size={16} /></span><div><strong>{disk.mountPoint || disk.name}</strong><small>{disk.fileSystem || t('volumes.fileSystemUnavailable')}{disk.removable ? ` · ${t('volumes.removable')}` : ''}</small></div><div className="volume-usage"><strong>{formatBytes(used, locale, t('system.unavailable'))} / {formatBytes(disk.totalBytes, locale, t('system.unavailable'))}</strong><div className="progress"><span style={{ width: `${percent(used, disk.totalBytes)}%` }} /></div></div></div>
            })}
          </div>
        </section>

        <div className="overview-side-stack"><BaselinePanel baseline={baseline} database={database} onAction={onBaselineAction} onOpenEvents={onOpenEvents} /></div>
      </div>

      {data.issues.length > 0 && <section className="collection-notice"><AlertTriangle size={17} /><div><strong>{t('collection.partial')}</strong>{data.issues.map((issue) => <p key={issue.component}>{t('collection.issue', { component: t(`collection.components.${issue.component}`, { defaultValue: issue.component }) })}</p>)}</div></section>}
      <p className="source-note">{t('collection.source', { source: data.source })}</p>
    </div>
  )
}
