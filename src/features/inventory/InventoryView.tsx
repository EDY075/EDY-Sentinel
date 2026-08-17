import { useEffect, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Boxes, CalendarDays, Database, Fingerprint, PackageSearch, RefreshCw, ScanSearch, ShieldCheck, ShieldQuestion } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import { evaluateSoftwareVulnerabilities, getSoftwareInventory, getSoftwareVulnerabilityDetail, getSoftwareVulnerabilitySummaries, refreshSoftwareInventory } from '../../lib/tauri'
import type { InstalledSoftware, ProductIdentityDetail, SoftwareVulnerabilityDetail, SoftwareVulnerabilitySummary, VulnerabilityMatch } from '../../types/inventory'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'
import { sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { filterInventory, formatInstallDate } from './inventory'
import type { InventoryFilter } from './inventory'
import { CveDetail } from './CveDetail'

let initialInventoryPromise: Promise<InstalledSoftware[]> | undefined
type InventoryRow = InstalledSoftware & {
  identityStatus: 'resolved' | 'unresolved'
  vulnerabilitySummary: SoftwareVulnerabilitySummary
  vulnerabilityCount: number
  highestCvss: number
  kevCount: number
  vulnerabilityStatus: string
}

const notEvaluated = (softwareId: string): SoftwareVulnerabilitySummary => ({
  softwareId, evaluationState: 'not_evaluated', confirmedCount: 0, possibleCount: 0,
  unresolvedCount: 0, notAffectedCount: 0, kevCount: 0,
})

function loadInitialInventory() {
  initialInventoryPromise ??= getSoftwareInventory()
    .then((items) => items.length ? items : refreshSoftwareInventory().then((snapshot) => snapshot.items))
    .catch((error) => {
      initialInventoryPromise = undefined
      throw error
    })
  return initialInventoryPromise
}

export function InventoryView({ initialSoftwareId }: { initialSoftwareId?: string }) {
  const { t, i18n } = useTranslation('inventory')
  const locale = i18n.resolvedLanguage ?? i18n.language
  const number = new Intl.NumberFormat(locale)
  const [items, setItems] = useState<InstalledSoftware[]>([])
  const [summaries, setSummaries] = useState<SoftwareVulnerabilitySummary[]>([])
  const [detail, setDetail] = useState<SoftwareVulnerabilityDetail>()
  const [selectedMatch, setSelectedMatch] = useState<VulnerabilityMatch>()
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [evaluating, setEvaluating] = useState(false)
  const [detailLoading, setDetailLoading] = useState(false)
  const [error, setError] = useState(false)
  const [evaluationError, setEvaluationError] = useState(false)
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<InventoryFilter>('all')
  const [sortKey, setSortKey] = useState<keyof InventoryRow>('displayName')
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc')
  const [selectedKey, setSelectedKey] = useState<string | undefined>(initialSoftwareId)

  useEffect(() => {
    let active = true
    Promise.all([loadInitialInventory(), getSoftwareVulnerabilitySummaries()])
      .then(([loaded, vulnerabilitySummaries]) => { if (active) { setItems(loaded); setSummaries(vulnerabilitySummaries); setError(false) } })
      .catch(() => { if (active) setError(true) })
      .finally(() => { if (active) setLoading(false) })
    return () => { active = false }
  }, [])

  useEffect(() => {
    if (!selectedKey) { setDetail(undefined); setSelectedMatch(undefined); return }
    let active = true
    setDetailLoading(true)
    getSoftwareVulnerabilityDetail(selectedKey)
      .then((value) => { if (active) setDetail(value) })
      .catch(() => { if (active) setEvaluationError(true) })
      .finally(() => { if (active) setDetailLoading(false) })
    return () => { active = false }
  }, [selectedKey])

  const summaryById = useMemo(() => new Map(summaries.map((summary) => [summary.softwareId, summary])), [summaries])
  const rows = useMemo(
    () => sortRows(filterInventory(items, query, filter).map((item) => {
      const vulnerabilitySummary = summaryById.get(item.softwareId) ?? notEvaluated(item.softwareId)
      return { ...item, identityStatus: item.normalizedIdentity.status, vulnerabilitySummary, vulnerabilityCount: vulnerabilitySummary.confirmedCount, highestCvss: vulnerabilitySummary.highestCvss ?? -1, kevCount: vulnerabilitySummary.kevCount, vulnerabilityStatus: vulnerabilitySummary.evaluationState }
    }), { key: sortKey, direction: sortDirection }),
    [filter, items, query, sortDirection, sortKey, summaryById],
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
    { key: 'vulnerabilityCount', label: t('columns.vulnerabilities'), width: '116px', render: (row) => <VulnerabilityCount summary={row.vulnerabilitySummary} t={t} number={number} /> },
    { key: 'highestCvss', label: t('columns.highestCvss'), width: '82px', priority: 'secondary', render: (row) => row.vulnerabilitySummary.highestCvss === undefined ? '—' : number.format(row.vulnerabilitySummary.highestCvss) },
    { key: 'kevCount', label: 'KEV', width: '62px', priority: 'tertiary', render: (row) => row.kevCount ? <Badge tone="danger">{number.format(row.kevCount)}</Badge> : '—' },
    { key: 'vulnerabilityStatus', label: t('columns.status'), width: '128px', render: (row) => <VulnerabilityStatus summary={row.vulnerabilitySummary} t={t} /> },
  ]

  const refresh = async () => {
    if (refreshing) return
    setRefreshing(true)
    setError(false)
    try {
      const snapshot = await refreshSoftwareInventory()
      setItems(snapshot.items)
      initialInventoryPromise = Promise.resolve(snapshot.items)
      setSummaries(await getSoftwareVulnerabilitySummaries())
    } catch {
      setError(true)
    } finally {
      setRefreshing(false)
    }
  }
  const evaluate = async (softwareId?: string) => {
    if (evaluating) return
    setEvaluating(true)
    setEvaluationError(false)
    try {
      await evaluateSoftwareVulnerabilities(softwareId)
      setSummaries(await getSoftwareVulnerabilitySummaries())
      if (selectedKey) setDetail(await getSoftwareVulnerabilityDetail(selectedKey))
    } catch {
      setEvaluationError(true)
    } finally {
      setEvaluating(false)
    }
  }
  const onSort = (key: keyof InventoryRow) => {
    if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc')
    else { setSortKey(key); setSortDirection('asc') }
  }

  if (loading) return <section className="operational-panel inventory-state" aria-busy="true"><RefreshCw className="spin" size={20} /><strong>{t('loading')}</strong></section>
  if (error && !items.length) return <section className="operational-panel inventory-state" role="alert"><Boxes size={22} /><strong>{t('error.title')}</strong><span>{t('error.description')}</span><button type="button" className="button" onClick={() => void refresh()}>{t('actions.retry')}</button></section>

  return <>
    <section className="operational-panel inventory-panel" aria-busy={refreshing || evaluating}>
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder={t('search')} meta={<><strong>{number.format(rows.length)}</strong> {t('ofTotal', { total: number.format(items.length) })}</>} />
      <div className="inventory-actions"><span role={evaluationError ? 'alert' : undefined}>{evaluationError ? t('error.evaluation') : error ? t('error.refresh') : t('snapshot.note')}</span><div><button type="button" className="button" onClick={() => void evaluate()} disabled={evaluating}><ScanSearch size={14} className={evaluating ? 'spin' : ''} />{t(evaluating ? 'actions.evaluating' : 'actions.evaluate')}</button><button type="button" className="button" onClick={() => void refresh()} disabled={refreshing}><RefreshCw size={14} className={refreshing ? 'spin' : ''} />{t(refreshing ? 'actions.refreshing' : 'actions.refresh')}</button></div></div>
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.softwareId} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={(row) => setSelectedKey(row.softwareId)} emptyTitle={t('empty.title')} emptyDescription={t('empty.description')} ariaLabel={t('ariaLabel')} />
    </section>
    <Drawer open={Boolean(selected)} title={selectedMatch ? selectedMatch.cveId : t('drawer.title')} wide={Boolean(selectedMatch)} onClose={() => { setSelectedKey(undefined); setSelectedMatch(undefined) }}>{selected && selectedMatch ? <CveDetail match={selectedMatch} softwareName={selected.displayName} locale={locale} onBack={() => setSelectedMatch(undefined)} /> : selected && <div className="drawer-content">
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
      <ProductIdentitySection identity={detail?.productIdentity} loading={detailLoading} t={t} />
      <DrawerSection icon={<ShieldQuestion size={15} />} title={t('drawer.vulnerabilities')}>
        {detailLoading ? <div className="inventory-vulnerability-boundary"><RefreshCw className="spin" size={14} />{t('drawer.loadingMatches')}</div> : detail ? <VulnerabilitySections detail={detail} t={t} number={number} onOpen={setSelectedMatch} onEvaluate={() => void evaluate(selected.softwareId)} evaluating={evaluating} /> : <div className="inventory-vulnerability-boundary">{t('drawer.matchingNotEvaluated')}</div>}
      </DrawerSection>
    </div>}</Drawer>
  </>
}

function ProductIdentitySection({ identity, loading, t }: { identity?: ProductIdentityDetail; loading: boolean; t: ReturnType<typeof useTranslation>['t'] }) {
  const unavailable = t('unavailable')
  const candidate = identity?.cpeCandidate ?? (identity?.status === 'ambiguous' ? identity.candidates.map(({ cpe }) => cpe).join('\n') : undefined)
  return <DrawerSection icon={<Database size={15} />} title={t('drawer.productIdentity')}>
    <Detail label={t('drawer.status')} value={loading ? t('drawer.loadingIdentity') : t(`productIdentityStatus.${identity?.status ?? 'unresolved'}`)} />
    <Detail label={t('drawer.canonicalVendor')} value={identity?.canonicalVendor ?? unavailable} />
    <Detail label={t('drawer.canonicalProduct')} value={identity?.canonicalProduct ?? unavailable} />
    <Detail label={t('drawer.normalizedVersion')} value={identity?.normalizedVersion ?? unavailable} mono />
    <Detail label={t('drawer.cpeCandidate')} value={candidate ?? unavailable} mono />
    <Detail label={t('drawer.resolutionMethod')} value={identity?.resolutionMethod ? t(`resolutionMethods.${identity.resolutionMethod}`) : unavailable} />
    <Detail label={t('drawer.confidence')} value={identity?.confidence ? t(`confidence.${identity.confidence}`) : unavailable} />
    {identity?.unresolvedReason && <Detail label={t('drawer.unresolvedReason')} value={t(`unresolvedReasons.${identity.unresolvedReason}`)} />}
    <Detail label={t('drawer.provenance')} value={identity?.provenance.join('\n') || unavailable} mono />
    <Detail label={t('drawer.resolverVersion')} value={identity ? `v${identity.resolverVersion}` : unavailable} mono />
  </DrawerSection>
}

function VulnerabilityCount({ summary, t, number }: { summary: SoftwareVulnerabilitySummary; t: ReturnType<typeof useTranslation>['t']; number: Intl.NumberFormat }) {
  if (summary.confirmedCount) return <span className="inventory-vuln-count"><strong>{number.format(summary.confirmedCount)}</strong><small>{t('matchStates.confirmed')}</small></span>
  if (summary.possibleCount) return <span className="inventory-vuln-count"><strong>{number.format(summary.possibleCount)}</strong><small>{t('matchStates.possible')}</small></span>
  return <span>—</span>
}

function VulnerabilityStatus({ summary, t }: { summary: SoftwareVulnerabilitySummary; t: ReturnType<typeof useTranslation>['t'] }) {
  const tone = summary.evaluationState === 'confirmed' ? 'danger' : summary.evaluationState === 'possible' ? 'warning' : summary.evaluationState === 'no_confirmed' || summary.evaluationState === 'not_affected' ? 'good' : 'neutral'
  return <Badge tone={tone}>{t(`evaluationStates.${summary.evaluationState}`)}</Badge>
}

function VulnerabilitySections({ detail, t, number, onOpen, onEvaluate, evaluating }: { detail: SoftwareVulnerabilityDetail; t: ReturnType<typeof useTranslation>['t']; number: Intl.NumberFormat; onOpen: (match: VulnerabilityMatch) => void; onEvaluate: () => void; evaluating: boolean }) {
  const confirmed = detail.matches.filter((match) => match.matchState === 'confirmed')
  const possible = detail.matches.filter((match) => match.matchState === 'possible')
  if (detail.summary.evaluationState === 'not_evaluated') return <div className="inventory-vulnerability-boundary"><ShieldQuestion size={17} /><span>{t('drawer.matchingNotEvaluated')}</span><button type="button" className="button" onClick={onEvaluate} disabled={evaluating}>{t(evaluating ? 'actions.evaluating' : 'actions.evaluateSoftware')}</button></div>
  return <div className="software-vulnerability-sections"><div className="software-vulnerability-summary"><ShieldCheck size={16} /><span><strong>{t(`evaluationStates.${detail.summary.evaluationState}`)}</strong><small>{t('drawer.engineVersion', { version: detail.summary.matchingEngineVersion })}</small></span></div><MatchList title={t('drawer.confirmedMatches')} empty={t('drawer.noConfirmed')} matches={confirmed} number={number} t={t} onOpen={onOpen} /><MatchList title={t('drawer.possibleMatches')} empty={t('drawer.noPossible')} matches={possible} number={number} t={t} onOpen={onOpen} /></div>
}

function MatchList({ title, empty, matches, number, t, onOpen }: { title: string; empty: string; matches: VulnerabilityMatch[]; number: Intl.NumberFormat; t: ReturnType<typeof useTranslation>['t']; onOpen: (match: VulnerabilityMatch) => void }) {
  return <section className="software-match-group"><h4>{title}</h4>{matches.length ? matches.map((match) => <button type="button" key={match.matchId} onClick={() => onOpen(match)}><span><strong>{match.cveId}</strong><small>{match.affectedRange}</small></span><span>{match.cvssScore === undefined ? '—' : number.format(match.cvssScore)}<small>{match.kev ? 'KEV' : t(`confidence.${match.confidence}`)}</small></span></button>) : <p>{empty}</p>}</section>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono pre-line' : undefined} title={value}>{value}</dd></div> }
