import { describe, expect, it } from 'vitest'
import type { SecurityEvent } from '../../types/baseline'
import { eventDetailSections, eventStatusLabel, eventTypeLabel, evidenceEntries, filterSecurityEvents } from './events'

const event = (overrides: Partial<SecurityEvent> = {}): SecurityEvent => ({ eventId: 'security-1', eventType: 'executable_first_seen', entityType: 'executable', entityKey: 'abc', title: 'New executable observed', timestamp: '2026-08-16T12:00:00Z', firstSeen: '2026-08-16T12:00:00Z', lastSeen: '2026-08-16T12:00:00Z', evidence: { process: 'sample.exe', path: 'C:\\sample.exe' }, baselineContext: { comparison: 'not previously observed' }, source: 'processes', baselineId: 'baseline-1', confidence: 'high', status: 'new', observationCount: 1, conditionActive: true, schemaVersion: 1, ...overrides })

describe('security event foundation UI models', () => {
  it('formats factual event and status labels without severity', () => {
    expect(eventTypeLabel('parent_child_first_seen')).toBe('Parent Child First Seen')
    expect(eventStatusLabel('acknowledged')).toBe('Acknowledged')
  })

  it('filters the security event table by status and evidence', () => {
    const values = [event(), event({ eventId: '2', status: 'resolved', title: 'Gateway changed', evidence: { after: '192.0.2.1' } })]
    expect(filterSecurityEvents(values, 'sample.exe', 'all')).toHaveLength(1)
    expect(filterSecurityEvents(values, '', 'resolved').map(({ eventId }) => eventId)).toEqual(['2'])
  })

  it('builds drawer sections from verifiable evidence and timeline fields', () => {
    const sections = eventDetailSections(event({ observationCount: 7 }))
    expect(sections.evidence).toContainEqual({ key: 'path', value: 'C:\\sample.exe' })
    expect(sections.baseline[0].value).toContain('not previously')
    expect(sections.timeline.observations).toBe(7)
    expect(sections.source.confidence).toBe('high')
  })

  it('handles an empty evidence object without fabricated values', () => {
    expect(evidenceEntries(null)).toEqual([])
    expect(evidenceEntries({ signer: null })).toEqual([{ key: 'signer', value: 'Unavailable' }])
  })
})
