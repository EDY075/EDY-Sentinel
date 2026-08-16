import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, Clock3, Cpu, Fingerprint, Network, UserRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer, Tooltip } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { ConnectionInfo, ProcessFilter, ProcessInfo } from '../../types/telemetry'
import { filterProcesses, sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { displayValue, formatBytes, formatDateTime, formatDurationFrom, formatPercent } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'
import { localizedDomainValue } from '../telemetry/presentation'

export function ProcessesView({ processes, connections, currentUser }: { processes: ProcessInfo[]; connections: ConnectionInfo[]; currentUser?: string }) {
  const { t, i18n } = useTranslation('processes')
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const unavailable = t('unavailable')
  const filters: Array<{ value: ProcessFilter; label: string }> = [
    { value: 'all', label: t('filters.all') }, { value: 'user', label: t('filters.user') }, { value: 'system', label: t('filters.system') }, { value: 'high-cpu', label: t('filters.highCpu') }, { value: 'high-memory', label: t('filters.highMemory') }, { value: 'no-company', label: t('filters.noCompany') }, { value: 'restricted', label: t('filters.restricted') },
  ]
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<ProcessFilter>('all')
  const [sortKey, setSortKey] = useState<keyof ProcessInfo>('cpuPercent')
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc')
  const [selectedKey, setSelectedKey] = useState<string>()

  const rows = useMemo(() => sortRows(filterProcesses(processes, query, filter, currentUser), { key: sortKey, direction: sortDirection }), [currentUser, filter, processes, query, sortDirection, sortKey])
  const selected = processes.find(({ key }) => key === selectedKey) ?? null
  const processConnections = useMemo(() => selected ? connections.filter(({ pid, active }) => active && pid === selected.pid) : [], [connections, selected])
  const columns: OperationalColumn<ProcessInfo>[] = [
    { key: 'name', label: t('columns.process'), width: 'minmax(170px, 1.5fr)', render: (row) => <span className="cell-primary"><Activity size={14} /> <span><strong>{row.name}</strong><small>{row.description ?? row.executablePath ?? t('executableMetadataUnavailable')}</small></span></span> },
    { key: 'pid', label: t('columns.pid'), width: '76px', render: (row) => <code>{row.pid}</code> },
    { key: 'cpuPercent', label: t('columns.cpu'), width: '82px', render: (row) => <Tooltip label={t('cpuTooltip')}><span className="metric-cell">{formatPercent(row.cpuPercent, locale, t('calculating'))}</span></Tooltip> },
    { key: 'memoryBytes', label: t('columns.memory'), width: '92px', render: (row) => formatBytes(row.memoryBytes, locale, unavailable) },
    { key: 'user', label: t('columns.user'), width: 'minmax(110px, 1fr)', priority: 'secondary', render: (row) => displayValue(row.user, unavailable) },
    { key: 'company', label: t('columns.company'), width: 'minmax(120px, 1.1fr)', priority: 'tertiary', render: (row) => displayValue(row.company, unavailable) },
    { key: 'accessStatus', label: t('columns.status'), width: '104px', render: (row) => <Badge tone={row.accessStatus.toLocaleLowerCase() === 'available' ? 'neutral' : 'warning'}>{localizedDomainValue(row.accessStatus, 'accessStatus', t)}</Badge> },
  ]

  const onSort = (key: keyof ProcessInfo) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }

  return <>
    <section className="operational-panel">
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder={t('search')} meta={<><strong>{number.format(rows.length)}</strong> {t('ofTotal', { total: number.format(processes.length) })}</>} />
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.key} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.key)} emptyTitle={t('empty.title')} emptyDescription={t('empty.description')} ariaLabel={t('ariaLabel')} />
    </section>
    <Drawer open={Boolean(selected)} title={t('drawer.title')} onClose={() => setSelectedKey(undefined)}>{selected && <div className="drawer-content">
      <DrawerSection icon={<Fingerprint size={15} />} title={t('drawer.identity')}><Detail label={t('drawer.name')} value={selected.name} /><Detail label="PID / PPID" value={`${selected.pid} / ${displayValue(selected.parentPid, unavailable)}`} mono /><Detail label={t('drawer.path')} value={displayValue(selected.executablePath, unavailable)} mono /><Detail label={t('drawer.commandLine')} value={displayValue(selected.commandLine, unavailable)} mono /></DrawerSection>
      <DrawerSection icon={<Cpu size={15} />} title={t('drawer.runtime')}><Detail label={t('drawer.cpuTotal')} value={formatPercent(selected.cpuPercent, locale, t('calculating'))} /><Detail label={t('drawer.coreEquivalent')} value={formatPercent(selected.coreEquivalentCpuPercent, locale, t('calculating'))} /><Detail label={t('drawer.memory')} value={formatBytes(selected.memoryBytes, locale, unavailable)} /><Detail label={t('drawer.threads')} value={displayValue(selected.threadCount, unavailable)} /><Detail label={t('drawer.started')} value={formatDateTime(selected.startTime, locale, unavailable)} /><Detail label={t('drawer.uptime')} value={formatDurationFrom(selected.startTime, locale, unavailable)} /></DrawerSection>
      <DrawerSection icon={<UserRound size={15} />} title={t('drawer.ownership')}><Detail label={t('drawer.user')} value={displayValue(selected.user, unavailable)} /><Detail label={t('drawer.company')} value={displayValue(selected.company, unavailable)} /><Detail label={t('drawer.digitalSignature')} value={localizedDomainValue(selected.signatureStatus, 'signatureStatus', t)} /><Detail label={t('drawer.signer')} value={displayValue(selected.signer, unavailable)} /><Detail label={t('drawer.access')} value={localizedDomainValue(selected.accessStatus, 'accessStatus', t)} /></DrawerSection>
      <DrawerSection icon={<Network size={15} />} title={t('drawer.network')}><Detail label={t('drawer.activeConnections')} value={number.format(processConnections.length)} />{processConnections.slice(0, 8).map((connection) => <Detail key={connection.key} label={connection.protocol.toUpperCase()} value={`${connection.localAddress}:${connection.localPort} → ${connection.remoteAddress ?? '*'}:${connection.remotePort ?? '*'}`} mono />)}</DrawerSection>
      <DrawerSection icon={<Clock3 size={15} />} title={t('drawer.technical')}><Detail label={t('drawer.architecture')} value={displayValue(selected.architecture, unavailable)} /><Detail label={t('drawer.firstSeen')} value={formatDateTime(selected.firstSeen, locale, unavailable)} /><Detail label={t('drawer.lastSeen')} value={formatDateTime(selected.lastSeen, locale, unavailable)} /><Detail label={t('drawer.observations')} value={number.format(selected.observationCount)} /></DrawerSection>
    </div>}</Drawer>
  </>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
