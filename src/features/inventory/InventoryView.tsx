import { useEffect, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Boxes, CalendarDays, Database, Fingerprint, PackageSearch, RefreshCw, ShieldQuestion } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import { getSoftwareInventory, refreshSoftwareInventory } from '../../lib/tauri'
import type { InstalledSoftware } from '../../types/inventory'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'
import { sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { filterInventory, formatInstallDate } from './inventory'
import type { InventoryFilter } from './inventory'

let initialInventoryPromise: Promise<InstalledSoftware[]> | undefined
type InventoryRow = InstalledSoftware & { identityStatus: 'resolved' | 'unresolved' }

function loadInitialInventory() {
  initialInventoryPromise ??= getSoftwareInventory()
    .then((items) => items.length ? items : refreshSoftwareInventory().then((snapshot) => snapshot.items))
    .catch((error) => {
      initialInventoryPromise = undefined
      throw error
    })
  return initialInventoryPromise
}

export function InventoryView() {
  const { t, i18n } = useTranslation('inventory')
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const [items, setItems] = useState<InstalledSoftware[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [error, setError] = useState(false)
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<InventoryFilter>('all')
  const [sortKey, setSortKey] = useState<keyof InventoryRow>('displayName')
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc')
  const [selectedKey, setSelectedKey] = useState<string>()

  useEffect(() => {
    let active = true
    loadInitialInventory()
      .then((loaded) => { if (active) { setItems(loaded); setError(false) } })
      .catch(() => { if (active) setError(true) })
      .finally(() => { if (active) setLoading(false) })
    return () => { active = false }
  }, [])

  const rows = useMemo(
    () => sortRows(filterInventory(items, query, filter).map((item) => ({ ...item, identityStatus: item.normalizedIdentity.status })), { key: sortKey, direction: sortDirection }),
    [filter, items, query, sortDirection, sortKey],
  )
  const selected = items.find(({ softwareId }) => softwareId === selectedKey) ?? null
  const filters: Array<{ value: InventoryFilter; label: string }> = [
    { value: 'all', label: t('filters.all') },
    { value: 'machine', label: t('filters.machine') },
    { value: 'user', label: t('filters.user') },
    { value: 'resolved', label: t('filters.resolved') },
    { value: 'unresolved', label: t('filters.unresolved') },
  ]
  const columns: OperationalColumn<InventoryRow>[] = [
    { key: 'displayName', label: t('columns.software'), width: 'minmax(190px, 1.7fr)', render: (row) => <span className="cell-primary"><PackageSearch size={14} /><span><strong>{row.displayName}</strong><small>{row.publisher ?? t('unavailable')}</small></span></span> },
    { key: 'displayVersion', label: t('columns.version'), width: 'minmax(90px, .75fr)', render: (row) => row.displayVersion ?? t('unavailable') },
    { key: 'publisher', label: t('columns.publisher'), width: 'minmax(130px, 1fr)', priority: 'secondary', render: (row) => row.publisher ?? t('unavailable') },
    { key: 'architecture', label: t('columns.architecture'), width: '88px', render: (row) => t(`architecture.${row.architecture}`) },
    { key: 'installScope', label: t('columns.scope'), width: '86px', priority: 'tertiary', render: (row) => t(`scope.${row.installScope}`) },
    { key: 'identityStatus', label: t('columns.status'), width: '104px', render: (row) => <Badge tone={row.identityStatus === 'resolved' ? 'good' : 'neutral'}>{t(`identityStatus.${row.identityStatus}`)}</Badge> },
  ]

  const refresh = async () => {
    if (refreshing) return
    setRefreshing(true)
    setError(false)
    try {
      const snapshot = await refreshSoftwareInventory()
      setItems(snapshot.items)
      initialInventoryPromise = Promise.resolve(snapshot.items)
    } catch {
      setError(true)
    } finally {
      setRefreshing(false)
    }
  }
  const onSort = (key: keyof InventoryRow) => {
    if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc')
    else { setSortKey(key); setSortDirection('asc') }
  }

  if (loading) return <section className="operational-panel inventory-state" aria-busy="true"><RefreshCw className="spin" size={20} /><strong>{t('loading')}</strong></section>
  if (error && !items.length) return <section className="operational-panel inventory-state" role="alert"><Boxes size={22} /><strong>{t('error.title')}</strong><span>{t('error.description')}</span><button type="button" className="button" onClick={() => void refresh()}>{t('actions.retry')}</button></section>

  return <>
    <section className="operational-panel inventory-panel" aria-busy={refreshing}>
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder={t('search')} meta={<><strong>{number.format(rows.length)}</strong> {t('ofTotal', { total: number.format(items.length) })}</>} />
      <div className="inventory-actions"><span>{error ? t('error.refresh') : t('snapshot.note')}</span><button type="button" className="button" onClick={() => void refresh()} disabled={refreshing}><RefreshCw size={14} className={refreshing ? 'spin' : ''} />{t(refreshing ? 'actions.refreshing' : 'actions.refresh')}</button></div>
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.softwareId} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.softwareId)} emptyTitle={t('empty.title')} emptyDescription={t('empty.description')} ariaLabel={t('ariaLabel')} />
    </section>
    <Drawer open={Boolean(selected)} title={t('drawer.title')} onClose={() => setSelectedKey(undefined)}>{selected && <div className="drawer-content">
      <DrawerSection icon={<Fingerprint size={15} />} title={t('drawer.presentation')}>
        <Detail label={t('drawer.name')} value={selected.displayName} />
        <Detail label={t('drawer.version')} value={selected.displayVersion ?? t('unavailable')} />
        <Detail label={t('drawer.publisher')} value={selected.publisher ?? t('unavailable')} />
        <Detail label={t('drawer.architecture')} value={t(`architecture.${selected.architecture}`)} />
        <Detail label={t('drawer.scope')} value={t(`scope.${selected.installScope}`)} />
      </DrawerSection>
      <DrawerSection icon={<CalendarDays size={15} />} title={t('drawer.installation')}>
        <Detail label={t('drawer.location')} value={selected.installLocation ?? t('unavailable')} mono />
        <Detail label={t('drawer.installDate')} value={formatInstallDate(selected.installDate, locale, t('unavailable'))} />
        <Detail label={t('drawer.sources')} value={selected.sources.join(', ')} mono />
        <Detail label={t('drawer.registryIdentity')} value={selected.registryIdentities.join('\n')} mono />
        <Detail label={t('drawer.productCode')} value={selected.productCode ?? t('unavailable')} mono />
      </DrawerSection>
      <DrawerSection icon={<Database size={15} />} title={t('drawer.normalizedIdentity')}>
        <Detail label={t('drawer.vendor')} value={localizedIdentity(selected.normalizedIdentity.vendor, t('unresolved'))} />
        <Detail label={t('drawer.product')} value={localizedIdentity(selected.normalizedIdentity.product, t('unresolved'))} />
        <Detail label={t('drawer.normalizedVersion')} value={localizedIdentity(selected.normalizedIdentity.version, t('unresolved'))} />
        <Detail label={t('drawer.status')} value={t(`identityStatus.${selected.normalizedIdentity.status}`)} />
      </DrawerSection>
      <DrawerSection icon={<ShieldQuestion size={15} />} title={t('drawer.vulnerabilityStatus')}>
        <div className="inventory-vulnerability-boundary"><dt>{t('drawer.vulnerabilityStatus')}</dt><dd>{t('drawer.matchingNotEvaluated')}</dd></div>
      </DrawerSection>
    </div>}</Drawer>
  </>
}

function localizedIdentity(value: string, unresolved: string) { return value === 'Unresolved' ? unresolved : value }
function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono pre-line' : undefined} title={value}>{value}</dd></div> }
