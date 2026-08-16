import { useCallback, useMemo, useState } from 'react'
import { Fingerprint, ShieldAlert } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import { formatDateTime, formatNumber } from '../../i18n'
import { getDetectionsPage } from '../../lib/tauri'
import type { Detection, DetectionCursor, DetectionEvidenceRecord, DetectionSeverity, DetectionStatus } from '../../types/detection'
import type { RuleDefinition } from '../../types/rules'
import { CursorPagination } from '../security/CursorPagination'
import { useCursorPager } from '../security/useCursorPager'
import { useDebouncedValue } from '../security/useDebouncedValue'
import { confidenceLabel, confidenceTone, detectionStatusLabel, localizedDetectionField, localizedEntityType, severityLabel, severityTone } from './detectionPresentation'
import { DetectionDrawer } from './DetectionDrawer'

const severities: Array<'all' | DetectionSeverity> = ['all', 'informational', 'low', 'medium', 'high', 'critical']
const statuses: Array<'all' | DetectionStatus> = ['all', 'new', 'investigating', 'acknowledged', 'resolved', 'ignored']
const toRfc3339 = (value: string) => value ? new Date(value).toISOString() : undefined

export function DetectionsView({ revision, rules, onStatusChange, onOpenSourceEvent }: {
  revision: number
  rules: RuleDefinition[]
  onStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
  onOpenSourceEvent: (evidence: DetectionEvidenceRecord) => void
}) {
  const { t } = useTranslation('detections')
  const { t: tRules } = useTranslation('rules')
  const [severity, setSeverity] = useState<'all' | DetectionSeverity>('all')
  const [status, setStatus] = useState<'all' | DetectionStatus>('all')
  const [category, setCategory] = useState('')
  const [ruleId, setRuleId] = useState('')
  const [entityType, setEntityType] = useState('')
  const [entityKey, setEntityKey] = useState('')
  const [from, setFrom] = useState('')
  const [to, setTo] = useState('')
  const [selectedKey, setSelectedKey] = useState<string>()
  const deferredEntityType = useDebouncedValue(entityType)
  const deferredEntityKey = useDebouncedValue(entityKey)
  const categories = useMemo(() => [...new Set(rules.map((rule) => rule.category))].sort(), [rules])
  const queryKey = JSON.stringify({ severity, status, category, ruleId, entityType: deferredEntityType, entityKey: deferredEntityKey, from, to })
  const load = useCallback((cursor?: DetectionCursor) => getDetectionsPage({
    severities: severity === 'all' ? undefined : [severity],
    statuses: status === 'all' ? undefined : [status],
    categories: category ? [category] : undefined,
    ruleIds: ruleId ? [ruleId] : undefined,
    entityType: deferredEntityType.trim() || undefined,
    entityKey: deferredEntityKey.trim() || undefined,
    from: toRfc3339(from),
    to: toRfc3339(to),
    cursor,
    limit: 50,
  }), [category, deferredEntityKey, deferredEntityType, from, ruleId, severity, status, to])
  const pager = useCursorPager({ load, queryKey, refreshKey: revision })
  const selected = pager.items.find(({ detectionId }) => detectionId === selectedKey) ?? null
  const selectedRule = selected ? rules.find(({ ruleId: id, version }) => id === selected.ruleId && version === selected.ruleVersion) : undefined
  const columns: OperationalColumn<Detection>[] = useMemo(() => [
    { key: 'lastDetectedAt', label: t('table.time'), width: '155px', render: (row) => <span className="metric-cell">{formatDateTime(row.lastDetectedAt)}</span> },
    { key: 'title', label: t('table.detection'), width: 'minmax(250px, 1.8fr)', render: (row) => { const definition = rules.find((rule) => rule.ruleId === row.ruleId && rule.version === row.ruleVersion); return <span className="cell-primary"><ShieldAlert size={14} /><span><strong>{localizedDetectionField(row.ruleId, 'title', row.title, t)}</strong><small>{row.ruleId} v{formatNumber(row.ruleVersion)} · {definition ? tRules(`categories.${definition.category}`, { defaultValue: definition.category }) : t('table.categoryUnavailable')}</small></span></span> } },
    { key: 'entityKey', label: t('table.entity'), width: 'minmax(170px, 1.15fr)', priority: 'secondary', render: (row) => <span className="cell-primary"><Fingerprint size={14} /><span><strong>{localizedEntityType(row.entityType, t)}</strong><small>{row.entityKey}</small></span></span> },
    { key: 'severity', label: t('table.severity'), width: '112px', render: (row) => <Badge tone={severityTone(row.severity)}>{severityLabel(row.severity, t)}</Badge> },
    { key: 'confidence', label: t('table.confidence'), width: '108px', priority: 'tertiary', render: (row) => <Badge tone={confidenceTone(row.confidence)}>{confidenceLabel(row.confidence, t)}</Badge> },
    { key: 'status', label: t('table.status'), width: '150px', render: (row) => <span className="detection-status-cell"><Badge>{detectionStatusLabel(row.status, t)}</Badge>{!row.conditionActive && <small>{t('table.conditionInactive')}</small>}</span> },
  ], [rules, t, tRules])

  return <>
    <section className="operational-panel">
      <div className="security-filter-bar security-filter-bar--detections">
        <div className="filter-chips" aria-label={t('filters.ariaLabel')}>{severities.map((item) => <button type="button" key={item} data-active={severity === item || undefined} aria-pressed={severity === item} onClick={() => setSeverity(item)}>{item === 'all' ? t('filters.allSeverity') : severityLabel(item, t)}</button>)}</div>
        <label><span>{t('filters.status')}</span><select value={status} onChange={(event) => setStatus(event.target.value as 'all' | DetectionStatus)}>{statuses.map((item) => <option value={item} key={item}>{item === 'all' ? t('filters.allStatus') : detectionStatusLabel(item, t)}</option>)}</select></label>
        <label><span>{t('filters.category')}</span><select value={category} onChange={(event) => setCategory(event.target.value)}><option value="">{t('filters.allCategories')}</option>{categories.map((item) => <option key={item} value={item}>{tRules(`categories.${item}`, { defaultValue: item })}</option>)}</select></label>
        <label><span>{t('filters.rule')}</span><select value={ruleId} onChange={(event) => setRuleId(event.target.value)}><option value="">{t('filters.allRules')}</option>{rules.map((rule) => <option value={rule.ruleId} key={`${rule.ruleId}-${rule.version}`}>{rule.ruleId}</option>)}</select></label>
        <details className="advanced-security-filters"><summary>{t('filters.more')}</summary><div><label><span>{t('filters.entityType')}</span><input value={entityType} onChange={(event) => setEntityType(event.target.value)} placeholder={t('filters.exactType')} /></label><label><span>{t('filters.entityKey')}</span><input value={entityKey} onChange={(event) => setEntityKey(event.target.value)} placeholder={t('filters.exactKey')} /></label><label><span>{t('filters.from')}</span><input type="datetime-local" value={from} onChange={(event) => setFrom(event.target.value)} /></label><label><span>{t('filters.to')}</span><input type="datetime-local" value={to} onChange={(event) => setTo(event.target.value)} /></label></div></details>
      </div>
      {pager.error && <div className="inline-error" role="alert">{t('table.loadError')}</div>}
      <OperationalTable rows={pager.items} columns={columns} rowKey={(row) => row.detectionId} selectedKey={selectedKey} sortKey="lastDetectedAt" sortDirection="desc" onSort={() => undefined} onSelect={(row) => setSelectedKey(row.detectionId)} sortable={false} pageKey={`${queryKey}-${pager.pageNumber}`} emptyTitle={pager.loading ? t('table.loading') : t('table.empty')} emptyDescription={t('table.emptyDescription')} ariaLabel={t('table.ariaLabel')} />
      <CursorPagination pageNumber={pager.pageNumber} itemCount={pager.items.length} nounKey="detections" hasMore={pager.hasMore} canPrevious={pager.canPrevious} loading={pager.loading} onPrevious={pager.previous} onNext={pager.next} onRetry={pager.reload} />
    </section>
    <DetectionDrawer detection={selected} rule={selectedRule} revision={revision} onClose={() => setSelectedKey(undefined)} onStatusChange={onStatusChange} onOpenSourceEvent={onOpenSourceEvent} />
  </>
}
