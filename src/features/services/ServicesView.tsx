import { useMemo, useState } from 'react'
import { Cog } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { ServiceFilter, ServiceInfo } from '../../types/telemetry'
import { filterServices, sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { displayValue } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'
import { localizedDomainValue } from '../telemetry/presentation'

export function ServicesView({ services }: { services: ServiceInfo[] }) {
  const { t, i18n } = useTranslation('services')
  const number = new Intl.NumberFormat(i18n.resolvedLanguage ?? i18n.language)
  const unavailable = t('unavailable')
  const filters: Array<{ value: ServiceFilter; label: string }> = [
    { value: 'all', label: t('filters.all') }, { value: 'running', label: t('filters.running') }, { value: 'stopped', label: t('filters.stopped') }, { value: 'automatic', label: t('filters.automatic') }, { value: 'manual', label: t('filters.manual') }, { value: 'disabled', label: t('filters.disabled') },
  ]
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<ServiceFilter>('all')
  const [sortKey, setSortKey] = useState<keyof ServiceInfo>('displayName')
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc')
  const [selectedKey, setSelectedKey] = useState<string>()
  const rows = useMemo(() => sortRows(filterServices(services, query, filter), { key: sortKey, direction: sortDirection }), [filter, query, services, sortDirection, sortKey])
  const columns: OperationalColumn<ServiceInfo>[] = [
    { key: 'displayName', label: t('columns.service'), width: 'minmax(220px, 1.8fr)', render: (row) => <span className="cell-primary"><Cog size={14} /><span><strong>{row.displayName}</strong><small>{row.serviceName} · {row.binaryPath ?? t('binaryPathUnavailable')}</small></span></span> },
    { key: 'status', label: t('columns.status'), width: '118px', render: (row) => <Badge tone={row.status.toLocaleLowerCase() === 'running' ? 'good' : row.status.toLocaleLowerCase().includes('pending') ? 'warning' : 'neutral'}>{localizedDomainValue(row.status, 'status', t)}</Badge> },
    { key: 'startupType', label: t('columns.startup'), width: '150px', render: (row) => localizedDomainValue(row.startupType, 'startupType', t) },
    { key: 'pid', label: t('columns.pid'), width: '82px', render: (row) => <code>{displayValue(row.pid, unavailable)}</code> },
    { key: 'account', label: t('columns.account'), width: 'minmax(150px, 1fr)', priority: 'secondary', render: (row) => displayValue(row.account, unavailable) },
  ]
  const onSort = (key: keyof ServiceInfo) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }

  return <section className="operational-panel">
    <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder={t('search')} meta={<><strong>{number.format(rows.length)}</strong> {t('ofTotal', { total: number.format(services.length) })}</>} />
    <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.key} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.key)} emptyTitle={t('empty.title')} emptyDescription={t('empty.description')} ariaLabel={t('ariaLabel')} />
  </section>
}
