import { useMemo, useState } from 'react'
import { Cog } from 'lucide-react'
import { Badge } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { ServiceFilter, ServiceInfo } from '../../types/telemetry'
import { filterServices, sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { displayValue } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'

const filters: Array<{ value: ServiceFilter; label: string }> = [
  { value: 'all', label: 'All' }, { value: 'running', label: 'Running' }, { value: 'stopped', label: 'Stopped' }, { value: 'automatic', label: 'Automatic' }, { value: 'manual', label: 'Manual' }, { value: 'disabled', label: 'Disabled' },
]

export function ServicesView({ services }: { services: ServiceInfo[] }) {
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<ServiceFilter>('all')
  const [sortKey, setSortKey] = useState<keyof ServiceInfo>('displayName')
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc')
  const [selectedKey, setSelectedKey] = useState<string>()
  const rows = useMemo(() => sortRows(filterServices(services, query, filter), { key: sortKey, direction: sortDirection }), [filter, query, services, sortDirection, sortKey])
  const columns: OperationalColumn<ServiceInfo>[] = [
    { key: 'displayName', label: 'Service', width: 'minmax(220px, 1.8fr)', render: (row) => <span className="cell-primary"><Cog size={14} /><span><strong>{row.displayName}</strong><small>{row.serviceName} · {row.binaryPath ?? 'Binary path unavailable'}</small></span></span> },
    { key: 'status', label: 'Status', width: '118px', render: (row) => <Badge tone={row.status.toLocaleLowerCase() === 'running' ? 'good' : row.status.toLocaleLowerCase().includes('pending') ? 'warning' : 'neutral'}>{row.status}</Badge> },
    { key: 'startupType', label: 'Startup', width: '150px', render: (row) => row.startupType },
    { key: 'pid', label: 'PID', width: '82px', render: (row) => <code>{displayValue(row.pid)}</code> },
    { key: 'account', label: 'Account', width: 'minmax(150px, 1fr)', priority: 'secondary', render: (row) => displayValue(row.account) },
  ]
  const onSort = (key: keyof ServiceInfo) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }

  return <section className="operational-panel">
    <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder="Search service name, display name, binary path, or account" meta={<><strong>{rows.length}</strong> of {services.length}</>} />
    <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.key} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.key)} emptyTitle="No services match this view" emptyDescription="Change the search or service-state filter." ariaLabel="Windows services" />
  </section>
}
