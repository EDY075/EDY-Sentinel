import type { ConnectionFilter, ConnectionInfo, ProcessFilter, ProcessInfo, ServiceFilter, ServiceInfo } from '../../types/telemetry'

export type SortDirection = 'asc' | 'desc'
export interface SortSpec<T> { key: keyof T; direction: SortDirection }

const includes = (value: unknown, query: string) => String(value ?? '').toLocaleLowerCase().includes(query)

export function sortRows<T>(rows: T[], sort: SortSpec<T>): T[] {
  const direction = sort.direction === 'asc' ? 1 : -1
  return [...rows].sort((left, right) => {
    const a = left[sort.key]
    const b = right[sort.key]
    if (a == null && b == null) return 0
    if (a == null) return 1
    if (b == null) return -1
    if (typeof a === 'number' && typeof b === 'number') return (a - b) * direction
    return String(a).localeCompare(String(b), undefined, { numeric: true, sensitivity: 'base' }) * direction
  })
}

export function filterProcesses(rows: ProcessInfo[], query: string, filter: ProcessFilter, currentUser?: string): ProcessInfo[] {
  const search = query.trim().toLocaleLowerCase()
  const searched = rows.filter((process) => {
    const searchable = !search || [process.name, process.pid, process.user, process.company, process.signer, process.executablePath].some((value) => includes(value, search))
    return searchable
  })
  if (filter === 'high-cpu') return searched.filter(({ cpuPercent }) => cpuPercent != null).sort((left, right) => (right.cpuPercent ?? 0) - (left.cpuPercent ?? 0)).slice(0, 50)
  if (filter === 'high-memory') return searched.sort((left, right) => right.memoryBytes - left.memoryBytes).slice(0, 50)
  return searched.filter((process) => {
    if (filter === 'user') return Boolean(currentUser && process.user?.toLocaleLowerCase().includes(currentUser.toLocaleLowerCase()))
    if (filter === 'system') return process.user?.toLocaleLowerCase().includes('system') ?? false
    if (filter === 'no-company') return !process.company
    if (filter === 'restricted') return process.accessStatus.toLocaleLowerCase() !== 'available'
    return true
  })
}

export function filterConnections(rows: ConnectionInfo[], query: string, filter: ConnectionFilter): ConnectionInfo[] {
  const search = query.trim().toLocaleLowerCase()
  return rows.filter((connection) => {
    const searchable = !search || [connection.processName, connection.pid, connection.protocol, connection.localAddress, connection.localPort, connection.remoteAddress, connection.remotePort, connection.state].some((value) => includes(value, search))
    if (!searchable) return false
    const protocol = connection.protocol.toLocaleLowerCase()
    const family = connection.ipVersion.toLocaleLowerCase()
    const state = connection.state?.toLocaleLowerCase() ?? ''
    if (filter === 'tcp' || filter === 'udp') return protocol.startsWith(filter)
    if (filter === 'ipv4' || filter === 'ipv6') return family === filter
    if (filter === 'established') return state === 'established'
    if (filter === 'listening') return state === 'listen' || state === 'listening'
    if (filter === 'other') return !['established', 'listen', 'listening'].includes(state)
    return true
  })
}

export function filterServices(rows: ServiceInfo[], query: string, filter: ServiceFilter): ServiceInfo[] {
  const search = query.trim().toLocaleLowerCase()
  return rows.filter((service) => {
    const searchable = !search || [service.serviceName, service.displayName, service.binaryPath, service.account].some((value) => includes(value, search))
    if (!searchable) return false
    const status = service.status.toLocaleLowerCase()
    const startup = service.startupType.toLocaleLowerCase()
    if (filter === 'running' || filter === 'stopped') return status === filter
    if (filter === 'automatic') return startup.startsWith('automatic')
    if (filter === 'manual' || filter === 'disabled') return startup === filter
    return true
  })
}
