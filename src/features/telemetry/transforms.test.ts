import { describe, expect, it } from 'vitest'
import type { ConnectionInfo, ProcessInfo, ServiceInfo } from '../../types/telemetry'
import { filterConnections, filterProcesses, filterServices, sortRows } from './transforms'
import { applyPausedState, deriveCollectorHealth } from './health'
import type { LiveTelemetrySnapshot } from '../../types/telemetry'
import { formatPercent } from './format'

const processFixture = (overrides: Partial<ProcessInfo>): ProcessInfo => ({ key: '10:1', name: 'app.exe', pid: 10, cpuPercent: 0, coreEquivalentCpuPercent: 0, memoryBytes: 1024, signatureStatus: 'unknown', accessStatus: 'available', firstSeen: '2026-01-01', lastSeen: '2026-01-01', observationCount: 1, active: true, ...overrides })
const connectionFixture = (overrides: Partial<ConnectionInfo>): ConnectionInfo => ({ key: 'tcp-1', protocol: 'tcp', ipVersion: 'ipv4', localAddress: '127.0.0.1', localPort: 5000, associationStatus: 'unresolved', firstSeen: '2026-01-01', lastSeen: '2026-01-01', observationCount: 1, active: true, ...overrides })
const serviceFixture = (overrides: Partial<ServiceInfo>): ServiceInfo => ({ key: 'service', serviceName: 'service', displayName: 'Service', status: 'Running', startupType: 'Automatic', firstSeen: '2026-01-01', lastSeen: '2026-01-01', observationCount: 1, active: true, ...overrides })
const collector = (id: string, status: 'healthy' | 'degraded' | 'failed', restrictedCount = 0) => ({ id, status, detail: 'Native collection', lastSuccess: '2026-01-01', lastAttempt: '2026-01-01', durationMs: 4, observationCount: 20, restrictedCount })

describe('operational telemetry transforms', () => {
  it('filters restricted processes without treating missing company metadata as a risk verdict', () => {
    const rows = [processFixture({ pid: 1, accessStatus: 'restricted' }), processFixture({ pid: 2, company: undefined })]
    expect(filterProcesses(rows, '', 'restricted').map(({ pid }) => pid)).toEqual([1])
    expect(filterProcesses(rows, '', 'no-company')).toHaveLength(2)
  })

  it('sorts numeric data without mutating the live snapshot', () => {
    const rows = [processFixture({ pid: 30, cpuPercent: null }), processFixture({ pid: 2, cpuPercent: 2.5 })]
    expect(sortRows(rows, { key: 'pid', direction: 'asc' }).map(({ pid }) => pid)).toEqual([2, 30])
    expect(rows.map(({ pid }) => pid)).toEqual([30, 2])
    expect(filterProcesses(rows, '', 'high-cpu').map(({ pid }) => pid)).toEqual([2])
  })

  it('filters real protocol, address family, and TCP state values', () => {
    const rows = [connectionFixture({ protocol: 'tcp', state: 'Established' }), connectionFixture({ key: 'udp-1', protocol: 'udp', ipVersion: 'ipv6' })]
    expect(filterConnections(rows, '', 'established')).toHaveLength(1)
    expect(filterConnections(rows, '', 'ipv6')[0]?.protocol).toBe('udp')
  })

  it('searches service identity and filters actual startup state', () => {
    const rows = [serviceFixture({ serviceName: 'W32Time', displayName: 'Windows Time' }), serviceFixture({ serviceName: 'ManualSvc', startupType: 'Manual' })]
    expect(filterServices(rows, 'windows time', 'all')[0]?.serviceName).toBe('W32Time')
    expect(filterServices(rows, '', 'manual')[0]?.serviceName).toBe('ManualSvc')
  })

  it('derives collector health independently from restricted metadata coverage', () => {
    const snapshot = { collectedAt: '2026-01-01', processes: [], connections: [], services: [], events: [], issues: [{ component: 'connections-tcp-ipv6', message: 'IPv6 table unavailable' }], collectors: [collector('processes', 'healthy', 17), collector('connections', 'degraded'), collector('services', 'healthy')] } satisfies LiveTelemetrySnapshot
    expect(deriveCollectorHealth(true, null, null).every(({ state }) => state === 'loading')).toBe(true)
    expect(deriveCollectorHealth(false, null, null, { system: 'Unavailable' }).find(({ key }) => key === 'system')?.state).toBe('failed')
    const health = deriveCollectorHealth(false, null, snapshot)
    expect(health.find(({ key }) => key === 'processes')?.state).toBe('healthy')
    expect(health.find(({ key }) => key === 'processes')?.restrictedCount).toBe(17)
    expect(health.find(({ key }) => key === 'network')?.state).toBe('degraded')
    expect(health.find(({ key }) => key === 'network')?.issue).toContain('IPv6')
    expect(applyPausedState(health, false).every(({ state }) => state === 'paused')).toBe(true)
  })

  it('formats CPU warm-up as calculating and uses normalized values afterwards', () => {
    expect(formatPercent(null)).toBe('Calculating')
    expect(formatPercent(8.333)).toBe('8.3%')
  })

  it('keeps company and verified signer as separate searchable facts', () => {
    const rows = [processFixture({ company: 'Example Company', signer: 'Example Code Signing CA' })]
    expect(filterProcesses(rows, 'company', 'all')).toHaveLength(1)
    expect(filterProcesses(rows, 'signing ca', 'all')).toHaveLength(1)
  })

  it('preserves factual connection association classifications', () => {
    const rows = [
      connectionFixture({ key: 'recent', processName: 'chrome.exe', associationStatus: 'recently_exited', processLastSeen: '2026-01-01' }),
      connectionFixture({ key: 'unknown', associationStatus: 'unresolved' }),
    ]
    expect(rows.map(({ associationStatus }) => associationStatus)).toEqual(['recently_exited', 'unresolved'])
  })
})
