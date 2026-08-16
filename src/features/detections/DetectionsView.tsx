import { useCallback, useMemo, useState } from 'react'
import { Fingerprint, ShieldAlert } from 'lucide-react'
import { Badge } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import { getDetectionsPage } from '../../lib/tauri'
import type { Detection, DetectionCursor, DetectionEvidenceRecord, DetectionSeverity, DetectionStatus } from '../../types/detection'
import type { RuleDefinition } from '../../types/rules'
import { CursorPagination } from '../security/CursorPagination'
import { useCursorPager } from '../security/useCursorPager'
import { useDebouncedValue } from '../security/useDebouncedValue'
import { formatDateTime } from '../telemetry/format'
import { confidenceLabel, confidenceTone, detectionStatusLabel, severityLabel, severityTone } from './detectionPresentation'
import { DetectionDrawer } from './DetectionDrawer'

const severities: Array<{ value: 'all' | DetectionSeverity; label: string }> = [
  { value: 'all', label: 'All severity' }, { value: 'informational', label: 'Informational' }, { value: 'low', label: 'Low' }, { value: 'medium', label: 'Medium' }, { value: 'high', label: 'High' }, { value: 'critical', label: 'Critical' },
]

const statuses: Array<'all' | DetectionStatus> = ['all', 'new', 'investigating', 'acknowledged', 'resolved', 'ignored']
const toRfc3339 = (value: string) => value ? new Date(value).toISOString() : undefined

export function DetectionsView({ revision, rules, onStatusChange, onOpenSourceEvent }: {
  revision: number
  rules: RuleDefinition[]
  onStatusChange: (detectionId: string, status: DetectionStatus) => Promise<void>
  onOpenSourceEvent: (evidence: DetectionEvidenceRecord) => void
}) {
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
    { key: 'lastDetectedAt', label: 'Time', width: '155px', render: (row) => <span className="metric-cell">{formatDateTime(row.lastDetectedAt)}</span> },
    { key: 'title', label: 'Detection', width: 'minmax(250px, 1.8fr)', render: (row) => <span className="cell-primary"><ShieldAlert size={14} /><span><strong>{row.title}</strong><small>{row.ruleId} v{row.ruleVersion} · {rules.find((rule) => rule.ruleId === row.ruleId && rule.version === row.ruleVersion)?.category ?? 'category unavailable'}</small></span></span> },
    { key: 'entityKey', label: 'Entity', width: 'minmax(170px, 1.15fr)', priority: 'secondary', render: (row) => <span className="cell-primary"><Fingerprint size={14} /><span><strong>{row.entityType}</strong><small>{row.entityKey}</small></span></span> },
    { key: 'severity', label: 'Severity', width: '112px', render: (row) => <Badge tone={severityTone(row.severity)}>{severityLabel(row.severity)}</Badge> },
    { key: 'confidence', label: 'Confidence', width: '108px', priority: 'tertiary', render: (row) => <Badge tone={confidenceTone(row.confidence)}>{confidenceLabel(row.confidence)}</Badge> },
    { key: 'status', label: 'Status', width: '150px', render: (row) => <span className="detection-status-cell"><Badge>{detectionStatusLabel(row.status)}</Badge>{!row.conditionActive && <small>Condition inactive</small>}</span> },
  ], [rules])

  return <>
    <section className="operational-panel">
      <div className="security-filter-bar security-filter-bar--detections">
        <div className="filter-chips" aria-label="Detection severity filters">{severities.map((item) => <button type="button" key={item.value} data-active={severity === item.value || undefined} aria-pressed={severity === item.value} onClick={() => setSeverity(item.value)}>{item.label}</button>)}</div>
        <label><span>Status</span><select value={status} onChange={(event) => setStatus(event.target.value as 'all' | DetectionStatus)}>{statuses.map((item) => <option value={item} key={item}>{item === 'all' ? 'All status' : detectionStatusLabel(item)}</option>)}</select></label>
        <label><span>Category</span><select value={category} onChange={(event) => setCategory(event.target.value)}><option value="">All categories</option>{categories.map((item) => <option key={item}>{item}</option>)}</select></label>
        <label><span>Rule</span><select value={ruleId} onChange={(event) => setRuleId(event.target.value)}><option value="">All rules</option>{rules.map((rule) => <option value={rule.ruleId} key={`${rule.ruleId}-${rule.version}`}>{rule.ruleId}</option>)}</select></label>
        <details className="advanced-security-filters"><summary>More filters</summary><div><label><span>Entity type</span><input value={entityType} onChange={(event) => setEntityType(event.target.value)} placeholder="Exact type" /></label><label><span>Entity key</span><input value={entityKey} onChange={(event) => setEntityKey(event.target.value)} placeholder="Exact key" /></label><label><span>From</span><input type="datetime-local" value={from} onChange={(event) => setFrom(event.target.value)} /></label><label><span>To</span><input type="datetime-local" value={to} onChange={(event) => setTo(event.target.value)} /></label></div></details>
      </div>
      {pager.error && <div className="inline-error" role="alert">Detections could not be loaded: {pager.error}</div>}
      <OperationalTable rows={pager.items} columns={columns} rowKey={(row) => row.detectionId} selectedKey={selectedKey} sortKey="lastDetectedAt" sortDirection="desc" onSort={() => undefined} onSelect={(row) => setSelectedKey(row.detectionId)} sortable={false} pageKey={`${queryKey}-${pager.pageNumber}`} emptyTitle={pager.loading ? 'Loading detections' : 'No detections under current coverage'} emptyDescription="Detections appear only when an enabled rule has sufficient corroborating evidence. Factual Events remain available separately." ariaLabel="Explainable detections" />
      <CursorPagination pageNumber={pager.pageNumber} itemCount={pager.items.length} noun="detections" hasMore={pager.hasMore} canPrevious={pager.canPrevious} loading={pager.loading} onPrevious={pager.previous} onNext={pager.next} onRetry={pager.reload} />
    </section>
    <DetectionDrawer detection={selected} rule={selectedRule} revision={revision} onClose={() => setSelectedKey(undefined)} onStatusChange={onStatusChange} onOpenSourceEvent={onOpenSourceEvent} />
  </>
}
