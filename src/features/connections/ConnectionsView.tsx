import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Clock3, MapPin, Network, Radio, Route } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { ConnectionFilter, ConnectionInfo } from '../../types/telemetry'
import { filterConnections, sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { displayValue, formatAge, formatDateTime } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'
import { localizedDomainValue } from '../telemetry/presentation'

const endpoint = (address: string | undefined, port: number | undefined, notApplicable: string) => address ? `${address}:${port ?? '*'}` : notApplicable

export function ConnectionsView({ connections }: { connections: ConnectionInfo[] }) {
  const { t, i18n } = useTranslation('connections')
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const unavailable = t('unavailable')
  const notApplicable = t('notApplicable')
  const filters: Array<{ value: ConnectionFilter; label: string }> = [
    { value: 'all', label: t('filters.all') }, { value: 'tcp', label: 'TCP' }, { value: 'udp', label: 'UDP' }, { value: 'ipv4', label: 'IPv4' }, { value: 'ipv6', label: 'IPv6' }, { value: 'established', label: t('filters.established') }, { value: 'listening', label: t('filters.listening') }, { value: 'other', label: t('filters.other') },
  ]
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<ConnectionFilter>('all')
  const [sortKey, setSortKey] = useState<keyof ConnectionInfo>('state')
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc')
  const [selectedKey, setSelectedKey] = useState<string>()
  const rows = useMemo(() => sortRows(filterConnections(connections, query, filter), { key: sortKey, direction: sortDirection }), [connections, filter, query, sortDirection, sortKey])
  const selected = connections.find(({ key }) => key === selectedKey) ?? null
  const processDetail = (row: ConnectionInfo) => {
    if (row.associationStatus === 'recently_exited') return t('process.exited', { age: formatAge(row.processLastSeen, locale, t('unknownAge')) })
    if (row.associationStatus === 'unresolved') return t('process.unresolvedIdentity')
    if (row.associationStatus === 'system_kernel') return t('process.systemKernel')
    if (row.associationStatus === 'not_applicable') return t('process.associationNotApplicable')
    return row.executablePath ?? t('process.associated')
  }
  const columns: OperationalColumn<ConnectionInfo>[] = [
    { key: 'processName', label: t('columns.process'), width: 'minmax(155px, 1.3fr)', render: (row) => <span className="cell-primary"><Network size={14} /><span><strong>{row.processName ?? (row.associationStatus === 'unresolved' ? t('process.unresolved') : t('process.notApplicable'))}</strong><small>{processDetail(row)}</small></span></span> },
    { key: 'pid', label: t('columns.pid'), width: '70px', render: (row) => <code>{displayValue(row.pid, unavailable)}</code> },
    { key: 'protocol', label: t('columns.protocol'), width: '86px', render: (row) => <Badge>{row.protocol.toUpperCase()} · {row.ipVersion.toUpperCase().slice(-1)}</Badge> },
    { key: 'localAddress', label: t('columns.local'), width: 'minmax(145px, 1.2fr)', render: (row) => <code>{endpoint(row.localAddress, row.localPort, notApplicable)}</code> },
    { key: 'remoteAddress', label: t('columns.remote'), width: 'minmax(145px, 1.2fr)', priority: 'secondary', render: (row) => <code>{endpoint(row.remoteAddress, row.remotePort, notApplicable)}</code> },
    { key: 'remotePort', label: t('columns.remotePort'), width: '90px', priority: 'tertiary', render: (row) => displayValue(row.remotePort, unavailable) },
    { key: 'state', label: t('columns.state'), width: '110px', render: (row) => <Badge tone={row.state?.toLocaleLowerCase() === 'established' ? 'good' : 'neutral'}>{row.state ? localizedDomainValue(row.state, 'tcpState', t) : (row.protocol === 'udp' ? t('state.stateless') : t('state.unavailable'))}</Badge> },
  ]
  const onSort = (key: keyof ConnectionInfo) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }

  return <>
    <section className="operational-panel">
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder={t('search')} meta={<><strong>{number.format(rows.length)}</strong> {t('ofTotal', { total: number.format(connections.length) })}</>} />
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.key} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.key)} emptyTitle={t('empty.title')} emptyDescription={t('empty.description')} ariaLabel={t('ariaLabel')} />
    </section>
    <Drawer open={Boolean(selected)} title={t('drawer.title')} onClose={() => setSelectedKey(undefined)}>{selected && <div className="drawer-content">
      <DrawerSection icon={<Network size={15} />} title={t('drawer.process')}><Detail label={t('drawer.process')} value={selected.processName ?? unavailable} /><Detail label={t('drawer.association')} value={localizedDomainValue(selected.associationStatus, 'associationStatus', t)} /><Detail label={t('drawer.processLastSeen')} value={selected.processLastSeen ? `${formatDateTime(selected.processLastSeen, locale, unavailable)} · ${formatAge(selected.processLastSeen, locale, t('unknownAge'))}` : notApplicable} /><Detail label="PID" value={displayValue(selected.pid, unavailable)} mono /><Detail label={t('drawer.executable')} value={displayValue(selected.executablePath, unavailable)} mono /></DrawerSection>
      <DrawerSection icon={<MapPin size={15} />} title={t('drawer.local')}><Detail label={t('drawer.address')} value={selected.localAddress} mono /><Detail label={t('drawer.port')} value={String(selected.localPort)} mono /><Detail label={t('drawer.protocol')} value={`${selected.protocol.toUpperCase()} · ${selected.ipVersion.toUpperCase()}`} /></DrawerSection>
      <DrawerSection icon={<Route size={15} />} title={t('drawer.remote')}><Detail label={t('drawer.address')} value={displayValue(selected.remoteAddress, unavailable)} mono /><Detail label={t('drawer.port')} value={displayValue(selected.remotePort, unavailable)} mono /></DrawerSection>
      <DrawerSection icon={<Radio size={15} />} title={t('drawer.state')}><Detail label={t('drawer.tcpState')} value={selected.state ? localizedDomainValue(selected.state, 'tcpState', t) : (selected.protocol === 'udp' ? t('state.udpNotApplicable') : unavailable)} /><Detail label={t('drawer.observation')} value={selected.active ? t('state.active') : t('state.closed')} /></DrawerSection>
      <DrawerSection icon={<Clock3 size={15} />} title={t('drawer.timeline')}><Detail label={t('drawer.firstSeen')} value={formatDateTime(selected.firstSeen, locale, unavailable)} /><Detail label={t('drawer.lastSeen')} value={formatDateTime(selected.lastSeen, locale, unavailable)} /><Detail label={t('drawer.observations')} value={number.format(selected.observationCount)} /><Detail label={t('drawer.trackingKey')} value={selected.key} mono /></DrawerSection>
    </div>}</Drawer>
  </>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
