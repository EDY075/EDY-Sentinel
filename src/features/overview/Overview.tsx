import { AlertTriangle, Box, Cpu, Database, Gauge, HardDrive, MemoryStick, MonitorCog, Network, RefreshCw, Router, Server, Wifi } from 'lucide-react'
import { Badge, EmptyState, Skeleton, StatusDot } from '../../components/ui/primitives'
import type { DatabaseStatus, SystemOverview } from '../../types/system'
import { formatBytes, formatUptime, percent } from './format'

interface OverviewProps {
  data: SystemOverview | null
  database: DatabaseStatus | null
  loading: boolean
  error: string | null
  onRefresh: () => void
}

function MetricCard({ icon, label, value, detail, progress }: { icon: React.ReactNode; label: string; value: string; detail: string; progress?: number }) {
  return (
    <article className="metric-card">
      <div className="metric-card__head"><span className="metric-icon">{icon}</span><span>{label}</span></div>
      <strong>{value}</strong>
      <p>{detail}</p>
      {progress !== undefined && <div className="progress" aria-label={`${progress}% used`}><span style={{ width: `${progress}%` }} /></div>}
    </article>
  )
}

function LoadingOverview() {
  return (
    <div className="overview-loading" aria-label="Collecting Windows telemetry">
      <div className="hero-card"><Skeleton className="skeleton--title" /><Skeleton className="skeleton--line" /></div>
      <div className="metrics-grid">{Array.from({ length: 4 }, (_, index) => <article className="metric-card" key={index}><Skeleton className="skeleton--icon" /><Skeleton className="skeleton--value" /><Skeleton className="skeleton--line" /></article>)}</div>
      <div className="details-grid"><div className="panel"><Skeleton className="skeleton--title" /><Skeleton className="skeleton--block" /></div><div className="panel"><Skeleton className="skeleton--title" /><Skeleton className="skeleton--block" /></div></div>
    </div>
  )
}

export function Overview({ data, database, loading, error, onRefresh }: OverviewProps) {
  if (loading) return <LoadingOverview />
  if (error) {
    return (
      <div className="error-state" role="alert">
        <span><AlertTriangle size={22} /></span>
        <div><strong>Windows telemetry could not be collected</strong><p>{error}</p></div>
        <button type="button" className="button" onClick={onRefresh}><RefreshCw size={16} /> Try again</button>
      </div>
    )
  }
  if (!data) return <EmptyState title="No system snapshot" description="Run a collection to create the first local snapshot." />

  const memoryUsed = percent(data.memory.usedBytes, data.memory.totalBytes)
  const diskTotal = data.disks.reduce((sum, disk) => sum + disk.totalBytes, 0)
  const diskUsed = data.disks.reduce((sum, disk) => sum + (disk.totalBytes - disk.availableBytes), 0)
  const osDetail = [data.operatingSystem.edition, data.operatingSystem.displayVersion, data.operatingSystem.build && `Build ${data.operatingSystem.build}`].filter(Boolean).join(' · ')

  return (
    <div className="overview">
      <section className="hero-card">
        <div>
          <div className="eyebrow"><StatusDot status={data.issues.length ? 'partial' : 'online'} /> Live Windows telemetry</div>
          <h1>{data.host.hostname}</h1>
          <p>{data.operatingSystem.name} · {data.host.architecture} · signed in as {data.host.username}</p>
        </div>
        <div className="hero-card__meta">
          <Badge tone={data.issues.length ? 'warning' : 'good'}>{data.issues.length ? 'Partial collection' : 'Local collectors active'}</Badge>
          <span>Updated {new Date(data.collectedAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span>
        </div>
      </section>

      <div className="metrics-grid">
        <MetricCard icon={<Cpu size={19} />} label="Processor" value={data.cpu.model} detail={`${data.cpu.physicalCores ?? '—'} physical · ${data.cpu.logicalCores} logical cores`} />
        <MetricCard icon={<MemoryStick size={19} />} label="Memory" value={`${formatBytes(data.memory.usedBytes)} used`} detail={`${formatBytes(data.memory.totalBytes)} installed`} progress={memoryUsed} />
        <MetricCard icon={<HardDrive size={19} />} label="Storage" value={`${formatBytes(diskUsed)} used`} detail={`${formatBytes(diskTotal)} across ${data.disks.length} volume${data.disks.length === 1 ? '' : 's'}`} progress={percent(diskUsed, diskTotal)} />
        <MetricCard icon={<Gauge size={19} />} label="Uptime" value={formatUptime(data.host.uptimeSeconds)} detail="Since the last Windows boot" />
      </div>

      <div className="details-grid">
        <section className="panel">
          <header className="panel__header"><div><MonitorCog size={18} /><span><strong>System profile</strong><small>Hardware and operating system</small></span></div><Badge>Real data</Badge></header>
          <dl className="detail-list">
            <div><dt><Server size={16} /> Windows</dt><dd>{data.operatingSystem.name}<small>{osDetail || 'Edition metadata unavailable'}</small></dd></div>
            <div><dt><Cpu size={16} /> CPU</dt><dd>{data.cpu.model}<small>{data.cpu.frequencyMhz ? `${data.cpu.frequencyMhz} MHz reported frequency` : 'Frequency unavailable'}</small></dd></div>
            <div><dt><Box size={16} /> GPU</dt><dd>{data.gpus[0]?.name ?? 'Unavailable'}<small>{data.gpus.length > 1 ? `${data.gpus.length} adapters detected` : data.gpus[0]?.driverVersion ? `Driver ${data.gpus[0].driverVersion}` : 'No additional details'}</small></dd></div>
          </dl>
        </section>

        <section className="panel">
          <header className="panel__header"><div><Network size={18} /><span><strong>Network posture</strong><small>Primary route and name resolution</small></span></div><Badge tone={data.network.primaryIpv4 ? 'good' : 'warning'}>{data.network.primaryIpv4 ? 'Connected' : 'Unavailable'}</Badge></header>
          <dl className="detail-list">
            <div><dt><Wifi size={16} /> Local IP</dt><dd>{data.network.primaryIpv4 ?? 'Unavailable'}<small>{data.network.primaryInterface ?? 'No primary interface detected'}</small></dd></div>
            <div><dt><Router size={16} /> Gateway</dt><dd>{data.network.gateways[0] ?? 'Unavailable'}<small>{data.network.gateways.length > 1 ? `${data.network.gateways.length} routes detected` : 'Primary route'}</small></dd></div>
            <div><dt><Database size={16} /> DNS</dt><dd>{data.network.dnsServers[0] ?? 'Unavailable'}<small>{data.network.dnsServers.slice(1).join(' · ') || 'No secondary resolver reported'}</small></dd></div>
          </dl>
        </section>
      </div>

      <div className="details-grid details-grid--lower">
        <section className="panel">
          <header className="panel__header"><div><HardDrive size={18} /><span><strong>Volumes</strong><small>Capacity reported by Windows</small></span></div><span className="panel-count">{data.disks.length}</span></header>
          <div className="volume-list">
            {data.disks.map((disk) => {
              const used = disk.totalBytes - disk.availableBytes
              return <div className="volume-row" key={`${disk.mountPoint}-${disk.name}`}><span className="volume-icon"><HardDrive size={16} /></span><div><strong>{disk.mountPoint || disk.name}</strong><small>{disk.fileSystem || 'File system unavailable'}{disk.removable ? ' · Removable' : ''}</small></div><div className="volume-usage"><strong>{formatBytes(used)} / {formatBytes(disk.totalBytes)}</strong><div className="progress"><span style={{ width: `${percent(used, disk.totalBytes)}%` }} /></div></div></div>
            })}
          </div>
        </section>

        <section className="panel score-panel">
          <header className="panel__header"><div><Gauge size={18} /><span><strong>Security Score</strong><small>Explainable assessment</small></span></div><Badge>Pending engine</Badge></header>
          <div className="score-empty"><span className="score-empty__ring"><Gauge size={26} /></span><div><strong>Security analysis engine not initialized</strong><p>No score is shown until the baseline and detection engines can produce an evidence-backed result.</p></div></div>
          <footer><StatusDot status={database?.writable ? 'online' : 'partial'} /><span>SQLite schema v{database?.schemaVersion ?? '—'} · {database?.writable ? 'Snapshots persist locally' : 'Persistence status unavailable'}</span></footer>
        </section>
      </div>

      {data.issues.length > 0 && <section className="collection-notice"><AlertTriangle size={17} /><div><strong>Some telemetry is unavailable</strong>{data.issues.map((issue) => <p key={issue.component}>{issue.component}: {issue.message}</p>)}</div></section>}
      <p className="source-note">Collection source: {data.source}. Values are captured locally; snapshot persistence status is shown above.</p>
    </div>
  )
}
