import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { Activity, Clock3, Database, FileSearch, Fingerprint, Radio, ShieldCheck } from 'lucide-react'
import { Badge, Drawer } from '../../components/ui/primitives'
import { OperationalTable } from '../../components/ui/OperationalTable'
import type { OperationalColumn } from '../../components/ui/OperationalTable'
import type { SecurityEvent, SecurityEventStatus } from '../../types/baseline'
import { formatDateTime } from '../telemetry/format'
import { OperationalToolbar } from '../telemetry/OperationalToolbar'
import { sortRows } from '../telemetry/transforms'
import type { SortDirection } from '../telemetry/transforms'
import { eventDetailSections, eventStatusLabel, eventTypeLabel, filterSecurityEvents } from './events'
import type { SecurityEventFilter } from './events'

const filters: Array<{ value: SecurityEventFilter; label: string }> = [
  { value: 'all', label: 'All' }, { value: 'new', label: 'New' }, { value: 'seen', label: 'Seen' }, { value: 'acknowledged', label: 'Acknowledged' }, { value: 'resolved', label: 'Resolved' }, { value: 'ignored', label: 'Ignored' },
]

const statusTone = (status: SecurityEventStatus) => status === 'new' ? 'warning' as const : status === 'acknowledged' ? 'good' as const : 'neutral' as const

export function SecurityEventsView({ events, onStatusChange }: { events: SecurityEvent[]; onStatusChange: (eventId: string, status: SecurityEventStatus) => Promise<void> }) {
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState<SecurityEventFilter>('all')
  const [sortKey, setSortKey] = useState<keyof SecurityEvent>('lastSeen')
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc')
  const [selectedKey, setSelectedKey] = useState<string>()
  const selected = events.find((event) => event.eventId === selectedKey)
  const rows = useMemo(() => sortRows(filterSecurityEvents(events, query, filter), { key: sortKey, direction: sortDirection }), [events, filter, query, sortDirection, sortKey])
  const columns: OperationalColumn<SecurityEvent>[] = [
    { key: 'lastSeen', label: 'Time', width: '155px', render: (row) => <span className="metric-cell">{formatDateTime(row.lastSeen)}</span> },
    { key: 'title', label: 'Event', width: 'minmax(230px, 1.7fr)', render: (row) => <span className="cell-primary"><FileSearch size={14} /><span><strong>{row.title}</strong><small>{eventTypeLabel(row.eventType)} · factual observation</small></span></span> },
    { key: 'entityKey', label: 'Entity', width: 'minmax(180px, 1.2fr)', priority: 'secondary', render: (row) => <span className="cell-primary"><Fingerprint size={14} /><span><strong>{row.entityType}</strong><small>{row.entityKey}</small></span></span> },
    { key: 'source', label: 'Source', width: '120px', render: (row) => row.source },
    { key: 'status', label: 'Status', width: '120px', render: (row) => <Badge tone={statusTone(row.status)}>{eventStatusLabel(row.status)}</Badge> },
  ]
  const onSort = (key: keyof SecurityEvent) => { if (sortKey === key) setSortDirection((value) => value === 'asc' ? 'desc' : 'asc'); else { setSortKey(key); setSortDirection('asc') } }
  const select = (event: SecurityEvent) => { setSelectedKey(event.eventId); if (event.status === 'new') void onStatusChange(event.eventId, 'seen') }

  return <>
    <section className="operational-panel">
      <OperationalToolbar query={query} onQueryChange={setQuery} filter={filter} onFilterChange={setFilter} options={filters} placeholder="Search event, entity, source, or evidence" meta={<><strong>{rows.length}</strong> of {events.length}</>} />
      <OperationalTable rows={rows} columns={columns} rowKey={(row) => row.eventId} selectedKey={selectedKey} sortKey={sortKey} sortDirection={sortDirection} onSort={onSort} onSelect={select} emptyTitle="No factual security events in this view" emptyDescription="A Learning baseline suppresses new-behavior events. Change filters or wait for a Ready baseline to observe a real difference." ariaLabel="Security events" />
    </section>
    <Drawer open={Boolean(selected)} title="Security event details" onClose={() => setSelectedKey(undefined)}>{selected && <EventDrawer event={selected} onStatusChange={onStatusChange} />}</Drawer>
  </>
}

function EventDrawer({ event, onStatusChange }: { event: SecurityEvent; onStatusChange: (eventId: string, status: SecurityEventStatus) => Promise<void> }) {
  const sections = eventDetailSections(event)
  return <div className="drawer-content">
    <DrawerSection icon={<Activity size={15} />} title="Summary"><Detail label="What happened" value={sections.summary.title} /><Detail label="Event type" value={sections.summary.eventType} /><Detail label="Status" value={eventStatusLabel(event.status)} /></DrawerSection>
    <DrawerSection icon={<Fingerprint size={15} />} title="Entity"><Detail label="Entity type" value={sections.entity.type} /><Detail label="Entity key" value={sections.entity.key} mono /></DrawerSection>
    <DrawerSection icon={<FileSearch size={15} />} title="Evidence">{sections.evidence.length ? sections.evidence.map((entry) => <Detail key={entry.key} label={entry.key} value={entry.value} mono={entry.value.includes('\\') || entry.value.includes(':')} />) : <Detail label="Evidence" value="No additional fields" />}</DrawerSection>
    <DrawerSection icon={<Database size={15} />} title="Baseline">{sections.baseline.map((entry) => <Detail key={entry.key} label={entry.key} value={entry.value} />)}<Detail label="Baseline ID" value={event.baselineId ?? 'Not applicable'} mono /></DrawerSection>
    <DrawerSection icon={<Clock3 size={15} />} title="Timeline"><Detail label="First seen" value={formatDateTime(sections.timeline.firstSeen)} /><Detail label="Last seen" value={formatDateTime(sections.timeline.lastSeen)} /><Detail label="Observations" value={String(sections.timeline.observations)} /></DrawerSection>
    <DrawerSection icon={<Radio size={15} />} title="Source"><Detail label="Collector" value={sections.source.collector} /><Detail label="Correlation confidence" value={sections.source.confidence ? `${sections.source.confidence} factual correlation` : 'Not assigned'} /><Detail label="Threat classification" value="Not performed" /></DrawerSection>
    <div className="event-actions" aria-label="Event status actions"><button type="button" className="button" onClick={() => void onStatusChange(event.eventId, 'acknowledged')}><ShieldCheck size={14} /> Acknowledge</button><button type="button" className="button" onClick={() => void onStatusChange(event.eventId, 'resolved')}>Resolve</button><button type="button" className="button" onClick={() => void onStatusChange(event.eventId, 'ignored')}>Ignore</button></div>
  </div>
}

function DrawerSection({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) { return <section className="drawer-section"><h3>{icon}{title}</h3><dl>{children}</dl></section> }
function Detail({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) { return <div><dt>{label}</dt><dd className={mono ? 'mono' : undefined} title={value}>{value}</dd></div> }
