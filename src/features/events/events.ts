import type { SecurityEvent, SecurityEventStatus } from '../../types/baseline'
import type { TFunction } from 'i18next'

export type SecurityEventFilter = 'all' | SecurityEventStatus

const fallbackTypeLabel = (value: string) => value.split('_').map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join(' ')
export const eventTypeLabel = (value: string, t?: TFunction<'events'>) => t?.(`eventTypes.${value}`, { defaultValue: fallbackTypeLabel(value) }) ?? fallbackTypeLabel(value)
export const eventTitle = (eventType: string, fallback: string, t: TFunction<'events'>) => t(`titles.${eventType}`, { defaultValue: fallback })
export const eventStatusLabel = (value: SecurityEventStatus, t?: TFunction<'events'>) => t?.(`status.${value}`, { defaultValue: value === 'acknowledged' ? 'Acknowledged' : value.charAt(0).toUpperCase() + value.slice(1) }) ?? (value === 'acknowledged' ? 'Acknowledged' : value.charAt(0).toUpperCase() + value.slice(1))
export const eventEntityTypeLabel = (value: string, t: TFunction<'events'>) => t(`entityTypes.${value}`, { defaultValue: value })
export const eventSourceLabel = (value: string, t: TFunction<'events'>) => t(`collectorSources.${value}`, { defaultValue: value })
export const eventEvidenceFieldLabel = (value: string, t: TFunction<'events'>) => t(`evidenceFields.${value}`, { defaultValue: value })
export const eventBaselineFieldLabel = (value: string, t: TFunction<'events'>) => t(`baselineFields.${value}`, { defaultValue: value })

export function filterSecurityEvents(events: SecurityEvent[], query: string, filter: SecurityEventFilter) {
  const needle = query.trim().toLocaleLowerCase()
  return events.filter((event) => {
    if (filter !== 'all' && event.status !== filter) return false
    if (!needle) return true
    return `${event.title} ${event.eventType} ${event.entityType} ${event.entityKey} ${event.source} ${JSON.stringify(event.evidence)}`.toLocaleLowerCase().includes(needle)
  })
}

export function evidenceEntries(value: Record<string, unknown> | null, unavailable = 'Unavailable') {
  if (!value) return []
  return Object.entries(value).map(([key, entry]) => ({
    key,
    value: entry == null || entry === 'Unavailable' ? unavailable : typeof entry === 'string' ? entry : JSON.stringify(entry),
  }))
}

export function eventDetailSections(event: SecurityEvent, t?: TFunction<'events'>) {
  const unavailable = t?.('unavailable') ?? 'Unavailable'
  return {
    summary: { title: t ? eventTitle(event.eventType, event.title, t) : event.title, eventType: eventTypeLabel(event.eventType, t) },
    entity: { type: event.entityType, key: event.entityKey },
    evidence: evidenceEntries(event.evidence, unavailable),
    baseline: evidenceEntries(event.baselineContext, unavailable),
    timeline: { firstSeen: event.firstSeen, lastSeen: event.lastSeen, observations: event.observationCount },
    source: { collector: event.source, confidence: event.confidence },
  }
}
