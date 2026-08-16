import type { SecurityEvent, SecurityEventStatus } from '../../types/baseline'

export type SecurityEventFilter = 'all' | SecurityEventStatus

export const eventTypeLabel = (value: string) => value.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ')
export const eventStatusLabel = (value: SecurityEventStatus) => value === 'acknowledged' ? 'Acknowledged' : value.charAt(0).toUpperCase() + value.slice(1)

export function filterSecurityEvents(events: SecurityEvent[], query: string, filter: SecurityEventFilter) {
  const needle = query.trim().toLocaleLowerCase()
  return events.filter((event) => {
    if (filter !== 'all' && event.status !== filter) return false
    if (!needle) return true
    return `${event.title} ${event.eventType} ${event.entityType} ${event.entityKey} ${event.source} ${JSON.stringify(event.evidence)}`.toLocaleLowerCase().includes(needle)
  })
}

export function evidenceEntries(value: Record<string, unknown> | null) {
  if (!value) return []
  return Object.entries(value).map(([key, entry]) => ({
    key,
    value: entry == null ? 'Unavailable' : typeof entry === 'string' ? entry : JSON.stringify(entry),
  }))
}

export function eventDetailSections(event: SecurityEvent) {
  return {
    summary: { title: event.title, eventType: eventTypeLabel(event.eventType) },
    entity: { type: event.entityType, key: event.entityKey },
    evidence: evidenceEntries(event.evidence),
    baseline: evidenceEntries(event.baselineContext),
    timeline: { firstSeen: event.firstSeen, lastSeen: event.lastSeen, observations: event.observationCount },
    source: { collector: event.source, confidence: event.confidence },
  }
}
