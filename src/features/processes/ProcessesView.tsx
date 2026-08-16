import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, Clock3, Cpu, Fingerprint, Network, UserRound } from 'lucide-react'
import { Badge, Drawer, Tooltip } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { ConnectionInfo, ProcessFilter, ProcessInfo } from '../../types/telemetry'
import { filterProcesses, sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { displayValue, formatBytes, formatDateTime, formatDurationFrom, formatPercent } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'

const filters: Array<{ value: ProcessFilter; label: string }> = [
  { value: 'all', label: 'All' }, { value: 'user', label: 'User' }, { value: 'system', label: 'System' }, { value: 'high-cpu', label: 'Higher CPU' }, { value: 'high-memory', label: 'Higher memory' }, { value: 'no-company', label: 'No company' }, { value: 'restricted', label: 'Restricted' },
]

export function ProcessesView({ processes, connections, currentUser }: { processes: ProcessInfo[]; connections: ConnectionInfo[]; currentUser?: string }) {
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<ProcessFilter>('all')
  const [sortKey, setSortKey] = useState<keyof ProcessInfo>('cpuPercent')
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc')
  const [selectedKey, setSelectedKey] = useState<string>()

  const rows = useMemo(() => sortRows(filterProcesses(processes, query, filter, currentUser), { key: sortKey, direction: sortDirection }), [currentUser, filter, processes, query, sortDirection, sortKey])
  const selected = processes.find(({ key }) => key === selectedKey) ?? null
  const processConnections = useMemo(() => selected ? connections.filter(({ pid, active }) => active && pid === selected.pid) : [], [connections, selected])
  const columns: OperationalColumn<ProcessInfo>[] = [
    { key: 'name', label: 'Process', width: 'minmax(170px, 1.5fr)', render: (row) => <span className="cell-primary"><Activity size={14} /> <span><strong>{row.name}</strong><small>{row.description ?? row.executablePath ?? 'Executable metadata unavailable'}</small></span></span> },
    { key: 'pid', label: 'PID', width: '76px', render: (row) => <code>{row.pid}</code> },
    { key: 'cpuPercent', label: 'CPU', width: '82px', render: (row) => <Tooltip label="CPU usage as a percentage of total logical processor capacity."><span className="metric-cell">{formatPercent(row.cpuPercent)}</span></Tooltip> },
    { key: 'memoryBytes', label: 'Memory', width: '92px', render: (row) => formatBytes(row.memoryBytes) },
    { key: 'user', label: 'User', width: 'minmax(110px, 1fr)', priority: 'secondary', render: (row) => displayValue(row.user) },
    { key: 'company', label: 'Company', width: 'minmax(120px, 1.1fr)', priority: 'tertiary', render: (row) => displayValue(row.company) },
    { key: 'accessStatus', label: 'Status', width: '104px', render: (row) => <Badge tone={row.accessStatus.toLocaleLowerCase() === 'available' ? 'neutral' : 'warning'}>{row.accessStatus}</Badge> },
  ]

  const onSort = (key: keyof ProcessInfo) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }

  return <>
    <section className="operational-panel">
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder="Search process, PID, user, company, signer, or path" meta={<><strong>{rows.length}</strong> of {processes.length}</>} />
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.key} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.key)} emptyTitle="No processes match this view" emptyDescription="Change the search or filter. Collection remains factual and local." ariaLabel="Windows processes" />
    </section>
    <Drawer open={Boolean(selected)} title="Process details" onClose={() => setSelectedKey(undefined)}>{selected && <div className="drawer-content">
      <DrawerSection icon={<Fingerprint size={15} />} title="Identity"><Detail label="Name" value={selected.name} /><Detail label="PID / PPID" value={`${selected.pid} / ${displayValue(selected.parentPid)}`} mono /><Detail label="Path" value={displayValue(selected.executablePath)} mono /><Detail label="Command line" value={displayValue(selected.commandLine)} mono /></DrawerSection>
      <DrawerSection icon={<Cpu size={15} />} title="Runtime"><Detail label="CPU · total capacity" value={formatPercent(selected.cpuPercent)} /><Detail label="Core-equivalent CPU" value={formatPercent(selected.coreEquivalentCpuPercent)} /><Detail label="Memory" value={formatBytes(selected.memoryBytes)} /><Detail label="Threads" value={displayValue(selected.threadCount)} /><Detail label="Started" value={formatDateTime(selected.startTime)} /><Detail label="Uptime" value={formatDurationFrom(selected.startTime)} /></DrawerSection>
      <DrawerSection icon={<UserRound size={15} />} title="Ownership"><Detail label="User" value={displayValue(selected.user)} /><Detail label="Company" value={displayValue(selected.company)} /><Detail label="Digital signature" value={displayValue(selected.signatureStatus)} /><Detail label="Signer" value={displayValue(selected.signer)} /><Detail label="Access" value={selected.accessStatus} /></DrawerSection>
      <DrawerSection icon={<Network size={15} />} title="Network"><Detail label="Active connections" value={String(processConnections.length)} />{processConnections.slice(0, 8).map((connection) => <Detail key={connection.key} label={connection.protocol.toUpperCase()} value={`${connection.localAddress}:${connection.localPort} → ${connection.remoteAddress ?? '*'}:${connection.remotePort ?? '*'}`} mono />)}</DrawerSection>
      <DrawerSection icon={<Clock3 size={15} />} title="Technical"><Detail label="Architecture" value={displayValue(selected.architecture)} /><Detail label="First seen" value={formatDateTime(selected.firstSeen)} /><Detail label="Last seen" value={formatDateTime(selected.lastSeen)} /><Detail label="Observations" value={String(selected.observationCount)} /></DrawerSection>
    </div>}</Drawer>
  </>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
