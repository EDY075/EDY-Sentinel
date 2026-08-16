import { useCallback, useEffect, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, Clock3, Database, FileSearch, Fingerprint, Radio, ShieldCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import { formatDateTime, formatNumber } from '../../i18n'
import { getSecurityEventHistory, getSecurityEventsPage } from '../../lib/tauri'
import type { SecurityEvent, SecurityEventCursor, SecurityEventHistoryCursor, SecurityEventStatus } from '../../types/baseline'
import { CursorPagination } from '../security/CursorPagination'
import { useCursorPager } from '../security/useCursorPager'
import { useDebouncedValue } from '../security/useDebouncedValue'
import { eventBaselineFieldLabel, eventDetailSections, eventEntityTypeLabel, eventEvidenceFieldLabel, eventSourceLabel, eventStatusLabel, eventTitle, eventTypeLabel } from './events'

const statuses: Array<'all' | SecurityEventStatus> = ['all', 'new', 'seen', 'acknowledged', 'resolved', 'ignored']

const statusTone = (status: SecurityEventStatus) => status === 'new' ? 'warning' as const : status === 'acknowledged' ? 'good' as const : 'neutral' as const

export interface SecurityEventTarget {
  requestId: number
  eventId: string
  eventType: string
  entityType: string
  entityKey: string
}

export function SecurityEventsView({ revision, requestedEvent, onStatusChange }: {
  revision: number
  requestedEvent?: SecurityEventTarget
  onStatusChange: (eventId: string, status: SecurityEventStatus) => Promise<void>
}) {
  const { t } = useTranslation('events')
  const [status, setStatus] = useState<'all' | SecurityEventStatus>('all')
  const [eventType, setEventType] = useState('')
  const [entityType, setEntityType] = useState('')
  const [entityKey, setEntityKey] = useState('')
  const [selectedKey, setSelectedKey] = useState<string>()
  const deferredEventType = useDebouncedValue(eventType)
  const deferredEntityType = useDebouncedValue(entityType)
  const deferredEntityKey = useDebouncedValue(entityKey)
  const queryKey = JSON.stringify({ status, eventType: deferredEventType, entityType: deferredEntityType, entityKey: deferredEntityKey })
  const load = useCallback((cursor?: SecurityEventCursor) => getSecurityEventsPage({
    statuses: status === 'all' ? undefined : [status],
    eventTypes: deferredEventType.trim() ? [deferredEventType.trim()] : undefined,
    entityType: deferredEntityType.trim() || undefined,
    entityKey: deferredEntityKey.trim() || undefined,
    cursor,
    limit: 50,
  }), [deferredEntityKey, deferredEntityType, deferredEventType, status])
  const pager = useCursorPager({ load, queryKey, refreshKey: revision })
  const selected = pager.items.find((event) => event.eventId === selectedKey)

  useEffect(() => {
    if (!requestedEvent) return
    setStatus('all')
    setEventType(requestedEvent.eventType)
    setEntityType(requestedEvent.entityType)
    setEntityKey(requestedEvent.entityKey)
    setSelectedKey(requestedEvent.eventId)
  }, [requestedEvent])

  useEffect(() => {
    if (requestedEvent && pager.items.some(({ eventId }) => eventId === requestedEvent.eventId)) setSelectedKey(requestedEvent.eventId)
  }, [pager.items, requestedEvent])

  const columns: OperationalColumn<SecurityEvent>[] = useMemo(() => [
    { key: 'lastSeen', label: t('table.time'), width: '155px', render: (row) => <span className="metric-cell">{formatDateTime(row.lastSeen)}</span> },
    { key: 'title', label: t('table.event'), width: 'minmax(230px, 1.7fr)', render: (row) => <span className="cell-primary"><FileSearch size={14} /><span><strong>{eventTitle(row.eventType, row.title, t)}</strong><small>{eventTypeLabel(row.eventType, t)} · {t('table.factualObservation')}</small></span></span> },
    { key: 'entityKey', label: t('table.entity'), width: 'minmax(180px, 1.2fr)', priority: 'secondary', render: (row) => <span className="cell-primary"><Fingerprint size={14} /><span><strong>{eventEntityTypeLabel(row.entityType, t)}</strong><small>{row.entityKey}</small></span></span> },
    { key: 'source', label: t('table.source'), width: '120px', priority: 'tertiary', render: (row) => eventSourceLabel(row.source, t) },
    { key: 'status', label: t('table.status'), width: '120px', render: (row) => <Badge tone={statusTone(row.status)}>{eventStatusLabel(row.status, t)}</Badge> },
  ], [t])

  const select = (event: SecurityEvent) => {
    setSelectedKey(event.eventId)
    if (event.status === 'new') void onStatusChange(event.eventId, 'seen')
  }

  return <>
    <section className="operational-panel">
      <div className="security-filter-bar">
        <div className="filter-chips" aria-label={t('filters.ariaLabel')}>{statuses.map((item) => <button type="button" key={item} data-active={status === item || undefined} aria-pressed={status === item} onClick={() => setStatus(item)}>{t(`status.${item}`)}</button>)}</div>
        <label><span>{t('filters.eventType')}</span><input value={eventType} onChange={(event) => setEventType(event.target.value)} placeholder={t('filters.exactEventType')} /></label>
        <label><span>{t('filters.entityType')}</span><input value={entityType} onChange={(event) => setEntityType(event.target.value)} placeholder={t('filters.exactEntityType')} /></label>
        <label className="security-filter-bar__entity"><span>{t('filters.entityKey')}</span><input value={entityKey} onChange={(event) => setEntityKey(event.target.value)} placeholder={t('filters.exactEntityKey')} /></label>
      </div>
      {pager.error && <div className="inline-error" role="alert">{t('table.loadError')}</div>}
      <OperationalTable rows={pager.items} columns={columns} rowKey={(row) => row.eventId} selectedKey={selectedKey} sortKey="lastSeen" sortDirection="desc" onSort={() => undefined} onSelect={select} sortable={false} pageKey={`${queryKey}-${pager.pageNumber}`} emptyTitle={pager.loading ? t('table.loading') : t('table.empty')} emptyDescription={t('table.emptyDescription')} ariaLabel={t('table.ariaLabel')} />
      <CursorPagination pageNumber={pager.pageNumber} itemCount={pager.items.length} nounKey="events" hasMore={pager.hasMore} canPrevious={pager.canPrevious} loading={pager.loading} onPrevious={pager.previous} onNext={pager.next} onRetry={pager.reload} />
    </section>
    <Drawer open={Boolean(selected)} title={t('drawer.title')} onClose={() => setSelectedKey(undefined)}>{selected && <EventDrawer event={selected} revision={revision} onStatusChange={onStatusChange} />}</Drawer>
  </>
}

function EventDrawer({ event, revision, onStatusChange }: { event: SecurityEvent; revision: number; onStatusChange: (eventId: string, status: SecurityEventStatus) => Promise<void> }) {
  const { t } = useTranslation('events')
  const sections = eventDetailSections(event, t)
  const [busy, setBusy] = useState(false)
  const loadHistory = useCallback((cursor?: SecurityEventHistoryCursor) => getSecurityEventHistory({ entityType: event.entityType, entityKey: event.entityKey, eventType: event.eventType, baselineId: event.baselineId, cursor, limit: 25 }), [event.baselineId, event.entityKey, event.entityType, event.eventType])
  const history = useCursorPager({ load: loadHistory, queryKey: event.eventId, refreshKey: revision })
  const changeStatus = async (status: SecurityEventStatus) => {
    setBusy(true)
    try { await onStatusChange(event.eventId, status) } finally { setBusy(false) }
  }
  return <div className="drawer-content">
    <DrawerSection icon={<Activity size={15} />} title={t('drawer.summary')}><Detail label={t('drawer.whatHappened')} value={sections.summary.title} /><Detail label={t('drawer.eventType')} value={sections.summary.eventType} /><Detail label={t('drawer.status')} value={eventStatusLabel(event.status, t)} /></DrawerSection>
    <DrawerSection icon={<Fingerprint size={15} />} title={t('drawer.entity')}><Detail label={t('drawer.entityType')} value={eventEntityTypeLabel(sections.entity.type, t)} /><Detail label={t('drawer.entityKey')} value={sections.entity.key} mono /></DrawerSection>
    <DrawerSection icon={<FileSearch size={15} />} title={t('drawer.evidence')}>{sections.evidence.length ? sections.evidence.map((entry) => <Detail key={entry.key} label={eventEvidenceFieldLabel(entry.key, t)} value={entry.value} mono={entry.value.includes('\\') || entry.value.includes(':')} />) : <Detail label={t('drawer.evidence')} value={t('drawer.noEvidence')} />}</DrawerSection>
    <DrawerSection icon={<Database size={15} />} title={t('drawer.baseline')}>{sections.baseline.map((entry) => <Detail key={entry.key} label={eventBaselineFieldLabel(entry.key, t)} value={entry.value} />)}<Detail label={t('drawer.baselineId')} value={event.baselineId ?? t('drawer.notApplicable')} mono /></DrawerSection>
    <DrawerSection icon={<Clock3 size={15} />} title={t('drawer.timeline')}><Detail label={t('drawer.firstSeen')} value={formatDateTime(sections.timeline.firstSeen)} /><Detail label={t('drawer.lastSeen')} value={formatDateTime(sections.timeline.lastSeen)} /><Detail label={t('drawer.observations')} value={formatNumber(sections.timeline.observations)} /><div className="provenance-list">{history.items.map((entry) => <span key={entry.historyId}><strong>{t(`transitions.${entry.transition}`, { defaultValue: entry.transition.replaceAll('_', ' ') })}</strong><small>{formatDateTime(entry.observedAt)} · {t('drawer.historyObservation', { count: entry.observationCount, formattedCount: formatNumber(entry.observationCount) })}</small></span>)}</div>{history.hasMore || history.canPrevious ? <CursorPagination pageNumber={history.pageNumber} itemCount={history.items.length} nounKey="history" hasMore={history.hasMore} canPrevious={history.canPrevious} loading={history.loading} onPrevious={history.previous} onNext={history.next} onRetry={history.reload} /> : null}</DrawerSection>
    <DrawerSection icon={<Radio size={15} />} title={t('drawer.source')}><Detail label={t('drawer.collector')} value={eventSourceLabel(sections.source.collector, t)} /><Detail label={t('drawer.correlationConfidence')} value={sections.source.confidence ? t('drawer.factualCorrelation', { confidence: t(`confidence.${sections.source.confidence}`, { defaultValue: sections.source.confidence }) }) : t('drawer.notAssigned')} /><Detail label={t('drawer.threatClassification')} value={t('drawer.notPerformed')} /></DrawerSection>
    <div className="event-actions" aria-label={t('drawer.actionsAriaLabel')}><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('acknowledged')}><ShieldCheck size={14} /> {t('drawer.acknowledge')}</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('resolved')}>{t('drawer.resolve')}</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('ignored')}>{t('drawer.ignore')}</button></div>
  </div>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
