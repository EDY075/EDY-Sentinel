import { useCallback, useEffect, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, Clock3, Database, FileSearch, Fingerprint, Radio, ShieldCheck } from 'lucide-react'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import { getSecurityEventHistory, getSecurityEventsPage } from '../../lib/tauri'
import type { SecurityEvent, SecurityEventCursor, SecurityEventHistoryCursor, SecurityEventStatus } from '../../types/baseline'
import { CursorPagination } from '../security/CursorPagination'
import { useCursorPager } from '../security/useCursorPager'
import { useDebouncedValue } from '../security/useDebouncedValue'
import { formatDateTime } from '../telemetry/format'
import { eventDetailSections, eventStatusLabel, eventTypeLabel } from './events'

const statuses: Array<{ value: 'all' | SecurityEventStatus; label: string }> = [
  { value: 'all', label: 'All' }, { value: 'new', label: 'New' }, { value: 'seen', label: 'Seen' }, { value: 'acknowledged', label: 'Acknowledged' }, { value: 'resolved', label: 'Resolved' }, { value: 'ignored', label: 'Ignored' },
]

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
    { key: 'lastSeen', label: 'Time', width: '155px', render: (row) => <span className="metric-cell">{formatDateTime(row.lastSeen)}</span> },
    { key: 'title', label: 'Event', width: 'minmax(230px, 1.7fr)', render: (row) => <span className="cell-primary"><FileSearch size={14} /><span><strong>{row.title}</strong><small>{eventTypeLabel(row.eventType)} · factual observation</small></span></span> },
    { key: 'entityKey', label: 'Entity', width: 'minmax(180px, 1.2fr)', priority: 'secondary', render: (row) => <span className="cell-primary"><Fingerprint size={14} /><span><strong>{row.entityType}</strong><small>{row.entityKey}</small></span></span> },
    { key: 'source', label: 'Source', width: '120px', priority: 'tertiary', render: (row) => row.source },
    { key: 'status', label: 'Status', width: '120px', render: (row) => <Badge tone={statusTone(row.status)}>{eventStatusLabel(row.status)}</Badge> },
  ], [])

  const select = (event: SecurityEvent) => {
    setSelectedKey(event.eventId)
    if (event.status === 'new') void onStatusChange(event.eventId, 'seen')
  }

  return <>
    <section className="operational-panel">
      <div className="security-filter-bar">
        <div className="filter-chips" aria-label="Event status filters">{statuses.map((item) => <button type="button" key={item.value} data-active={status === item.value || undefined} aria-pressed={status === item.value} onClick={() => setStatus(item.value)}>{item.label}</button>)}</div>
        <label><span>Event type</span><input value={eventType} onChange={(event) => setEventType(event.target.value)} placeholder="Exact event type" /></label>
        <label><span>Entity type</span><input value={entityType} onChange={(event) => setEntityType(event.target.value)} placeholder="Exact type" /></label>
        <label className="security-filter-bar__entity"><span>Entity key</span><input value={entityKey} onChange={(event) => setEntityKey(event.target.value)} placeholder="Exact entity key" /></label>
      </div>
      {pager.error && <div className="inline-error" role="alert">Events could not be loaded: {pager.error}</div>}
      <OperationalTable rows={pager.items} columns={columns} rowKey={(row) => row.eventId} selectedKey={selectedKey} sortKey="lastSeen" sortDirection="desc" onSort={() => undefined} onSelect={select} sortable={false} pageKey={`${queryKey}-${pager.pageNumber}`} emptyTitle={pager.loading ? 'Loading factual security events' : 'No factual security events in this view'} emptyDescription="A Learning baseline suppresses new-behavior events. Change the exact filters or wait for a Ready baseline." ariaLabel="Security events" />
      <CursorPagination pageNumber={pager.pageNumber} itemCount={pager.items.length} noun="events" hasMore={pager.hasMore} canPrevious={pager.canPrevious} loading={pager.loading} onPrevious={pager.previous} onNext={pager.next} onRetry={pager.reload} />
    </section>
    <Drawer open={Boolean(selected)} title="Security event details" onClose={() => setSelectedKey(undefined)}>{selected && <EventDrawer event={selected} revision={revision} onStatusChange={onStatusChange} />}</Drawer>
  </>
}

function EventDrawer({ event, revision, onStatusChange }: { event: SecurityEvent; revision: number; onStatusChange: (eventId: string, status: SecurityEventStatus) => Promise<void> }) {
  const sections = eventDetailSections(event)
  const [busy, setBusy] = useState(false)
  const loadHistory = useCallback((cursor?: SecurityEventHistoryCursor) => getSecurityEventHistory({ entityType: event.entityType, entityKey: event.entityKey, eventType: event.eventType, baselineId: event.baselineId, cursor, limit: 25 }), [event.baselineId, event.entityKey, event.entityType, event.eventType])
  const history = useCursorPager({ load: loadHistory, queryKey: event.eventId, refreshKey: revision })
  const changeStatus = async (status: SecurityEventStatus) => {
    setBusy(true)
    try { await onStatusChange(event.eventId, status) } finally { setBusy(false) }
  }
  return <div className="drawer-content">
    <DrawerSection icon={<Activity size={15} />} title="Summary"><Detail label="What happened" value={sections.summary.title} /><Detail label="Event type" value={sections.summary.eventType} /><Detail label="Status" value={eventStatusLabel(event.status)} /></DrawerSection>
    <DrawerSection icon={<Fingerprint size={15} />} title="Entity"><Detail label="Entity type" value={sections.entity.type} /><Detail label="Entity key" value={sections.entity.key} mono /></DrawerSection>
    <DrawerSection icon={<FileSearch size={15} />} title="Evidence">{sections.evidence.length ? sections.evidence.map((entry) => <Detail key={entry.key} label={entry.key} value={entry.value} mono={entry.value.includes('\\') || entry.value.includes(':')} />) : <Detail label="Evidence" value="No additional fields" />}</DrawerSection>
    <DrawerSection icon={<Database size={15} />} title="Baseline">{sections.baseline.map((entry) => <Detail key={entry.key} label={entry.key} value={entry.value} />)}<Detail label="Baseline ID" value={event.baselineId ?? 'Not applicable'} mono /></DrawerSection>
    <DrawerSection icon={<Clock3 size={15} />} title="Timeline"><Detail label="First seen" value={formatDateTime(sections.timeline.firstSeen)} /><Detail label="Last seen" value={formatDateTime(sections.timeline.lastSeen)} /><Detail label="Observations" value={String(sections.timeline.observations)} /><div className="provenance-list">{history.items.map((entry) => <span key={entry.historyId}><strong>{entry.transition.replaceAll('_', ' ')}</strong><small>{formatDateTime(entry.observedAt)} · observation {entry.observationCount}</small></span>)}</div>{history.hasMore || history.canPrevious ? <CursorPagination pageNumber={history.pageNumber} itemCount={history.items.length} noun="history records" hasMore={history.hasMore} canPrevious={history.canPrevious} loading={history.loading} onPrevious={history.previous} onNext={history.next} onRetry={history.reload} /> : null}</DrawerSection>
    <DrawerSection icon={<Radio size={15} />} title="Source"><Detail label="Collector" value={sections.source.collector} /><Detail label="Correlation confidence" value={sections.source.confidence ? `${sections.source.confidence} factual correlation` : 'Not assigned'} /><Detail label="Threat classification" value="Not performed" /></DrawerSection>
    <div className="event-actions" aria-label="Event status actions"><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('acknowledged')}><ShieldCheck size={14} /> Acknowledge</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('resolved')}>Resolve</button><button type="button" className="button" disabled={busy} onClick={() => void changeStatus('ignored')}>Ignore</button></div>
  </div>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
