import { describe, expect, it } from 'vitest'
import type { ConnectionInfo, ProcessInfo, ServiceInfo } from '../../types/telemetry'
import { filterConnections, filterProcesses, filterServices, sortRows } from './transforms'
import { deriveCollectorHealth } from './health'
import type { LiveTelemetrySnapshot } from '../../types/telemetry'

const processFixture = (overrides: Partial<ProcessInfo>): ProcessInfo => ({ key: '10:1', name: 'app.exe', pid: 10, cpuPercent: 0, memoryBytes: 1024, signatureStatus: 'unavailable', accessStatus: 'available', firstSeen: '2026-01-01', lastSeen: '2026-01-01', observationCount: 1, active: true, ...overrides })
const connectionFixture = (overrides: Partial<ConnectionInfo>): ConnectionInfo => ({ key: 'tcp-1', protocol: 'tcp', ipVersion: 'ipv4', localAddress: '127.0.0.1', localPort: 5000, firstSeen: '2026-01-01', lastSeen: '2026-01-01', observationCount: 1, active: true, ...overrides })
const serviceFixture = (overrides: Partial<ServiceInfo>): ServiceInfo => ({ key: 'service', serviceName: 'service', displayName: 'Service', status: 'Running', startupType: 'Automatic', firstSeen: '2026-01-01', lastSeen: '2026-01-01', observationCount: 1, active: true, ...overrides })

describe('operational telemetry transforms', () => {
  it('filters restricted processes without treating missing publishers as a risk verdict', () => {
    const rows = [processFixture({ pid: 1, accessStatus: 'restricted' }), processFixture({ pid: 2, publisher: undefined })]
    expect(filterProcesses(rows, '', 'restricted').map(({ pid }) => pid)).toEqual([1])
    expect(filterProcesses(rows, '', 'no-publisher')).toHaveLength(2)
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

  it('derives loading, active, and partial collector states from backend facts', () => {
    const snapshot = { collectedAt: '2026-01-01', processes: [], connections: [], services: [], events: [], issues: [{ component: 'connections-tcp-ipv6', message: 'IPv6 table unavailable' }], collectors: [{ id: 'processes', status: 'active', detail: 'Native collection', collectedAt: '2026-01-01' }, { id: 'connections', status: 'partial', detail: 'Partial collection', collectedAt: '2026-01-01' }, { id: 'services', status: 'active', detail: 'Native collection', collectedAt: '2026-01-01' }] } satisfies LiveTelemetrySnapshot
    expect(deriveCollectorHealth(true, null, null).every(({ state }) => state === 'loading')).toBe(true)
    expect(deriveCollectorHealth(false, null, null, { system: 'Unavailable' }).find(({ key }) => key === 'system')?.state).toBe('error')
    const health = deriveCollectorHealth(false, null, snapshot)
    expect(health.find(({ key }) => key === 'processes')?.state).toBe('active')
    expect(health.find(({ key }) => key === 'network')?.state).toBe('partial')
    expect(health.find(({ key }) => key === 'network')?.issue).toContain('IPv6')
  })
})
