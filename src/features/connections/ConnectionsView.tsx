import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Clock3, MapPin, Network, Radio, Route } from 'lucide-react'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { ConnectionFilter, ConnectionInfo } from '../../types/telemetry'
import { filterConnections, sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { displayValue, formatDateTime } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'

const filters: Array<{ value: ConnectionFilter; label: string }> = [
  { value: 'all', label: 'All' }, { value: 'tcp', label: 'TCP' }, { value: 'udp', label: 'UDP' }, { value: 'ipv4', label: 'IPv4' }, { value: 'ipv6', label: 'IPv6' }, { value: 'established', label: 'Established' }, { value: 'listening', label: 'Listening' }, { value: 'other', label: 'Other states' },
]

const endpoint = (address?: string, port?: number) => address ? `${address}:${port ?? '*'}` : 'Not applicable'

export function ConnectionsView({ connections }: { connections: ConnectionInfo[] }) {
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<ConnectionFilter>('all')
  const [sortKey, setSortKey] = useState<keyof ConnectionInfo>('state')
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc')
  const [selectedKey, setSelectedKey] = useState<string>()
  const rows = useMemo(() => sortRows(filterConnections(connections, query, filter), { key: sortKey, direction: sortDirection }), [connections, filter, query, sortDirection, sortKey])
  const selected = connections.find(({ key }) => key === selectedKey) ?? null
  const columns: OperationalColumn<ConnectionInfo>[] = [
    { key: 'processName', label: 'Process', width: 'minmax(135px, 1.2fr)', render: (row) => <span className="cell-primary"><Network size={14} /><span><strong>{row.processName ?? 'Unassociated'}</strong><small>{row.executablePath ?? 'No process metadata'}</small></span></span> },
    { key: 'pid', label: 'PID', width: '70px', render: (row) => <code>{displayValue(row.pid)}</code> },
    { key: 'protocol', label: 'Protocol', width: '86px', render: (row) => <Badge>{row.protocol.toUpperCase()} · {row.ipVersion.toUpperCase().slice(-1)}</Badge> },
    { key: 'localAddress', label: 'Local', width: 'minmax(145px, 1.2fr)', render: (row) => <code>{endpoint(row.localAddress, row.localPort)}</code> },
    { key: 'remoteAddress', label: 'Remote', width: 'minmax(145px, 1.2fr)', priority: 'secondary', render: (row) => <code>{endpoint(row.remoteAddress, row.remotePort)}</code> },
    { key: 'remotePort', label: 'Remote port', width: '90px', priority: 'tertiary', render: (row) => displayValue(row.remotePort) },
    { key: 'state', label: 'State', width: '110px', render: (row) => <Badge tone={row.state?.toLocaleLowerCase() === 'established' ? 'good' : 'neutral'}>{row.state ?? (row.protocol === 'udp' ? 'Stateless' : 'Unavailable')}</Badge> },
  ]
  const onSort = (key: keyof ConnectionInfo) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }

  return <>
    <section className="operational-panel">
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder="Search process, PID, address, port, or state" meta={<><strong>{rows.length}</strong> of {connections.length}</>} />
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.key} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.key)} emptyTitle="No connections match this view" emptyDescription="Change the search or protocol filter. No external enrichment is applied." ariaLabel="Active Windows network connections" />
    </section>
    <Drawer open={Boolean(selected)} title="Connection details" onClose={() => setSelectedKey(undefined)}>{selected && <div className="drawer-content">
      <DrawerSection icon={<Network size={15} />} title="Process"><Detail label="Process" value={selected.processName ?? 'Unassociated'} /><Detail label="PID" value={displayValue(selected.pid)} mono /><Detail label="Executable" value={displayValue(selected.executablePath)} mono /></DrawerSection>
      <DrawerSection icon={<MapPin size={15} />} title="Local"><Detail label="Address" value={selected.localAddress} mono /><Detail label="Port" value={String(selected.localPort)} mono /><Detail label="Protocol" value={`${selected.protocol.toUpperCase()} · ${selected.ipVersion.toUpperCase()}`} /></DrawerSection>
      <DrawerSection icon={<Route size={15} />} title="Remote"><Detail label="Address" value={displayValue(selected.remoteAddress)} mono /><Detail label="Port" value={displayValue(selected.remotePort)} mono /></DrawerSection>
      <DrawerSection icon={<Radio size={15} />} title="State"><Detail label="TCP state" value={selected.state ?? (selected.protocol === 'udp' ? 'Not applicable to UDP' : 'Unavailable')} /><Detail label="Observation" value={selected.active ? 'Active' : 'Closed'} /></DrawerSection>
      <DrawerSection icon={<Clock3 size={15} />} title="Timeline"><Detail label="First seen" value={formatDateTime(selected.firstSeen)} /><Detail label="Last seen" value={formatDateTime(selected.lastSeen)} /><Detail label="Observations" value={String(selected.observationCount)} /><Detail label="Tracking key" value={selected.key} mono /></DrawerSection>
    </div>}</Drawer>
  </>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
