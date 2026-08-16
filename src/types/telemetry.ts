export type CollectorKey = 'system' | 'processes' | 'network' | 'services'
export type CollectorState = 'active' | 'partial' | 'error' | 'loading'

export interface TelemetryIssue {
  component: string
  message: string
}

export interface ProcessInfo {
  key: string
  name: string
  pid: number
  parentPid?: number
  user?: string
  executablePath?: string
  commandLine?: string
  cpuPercent: number | null
  memoryBytes: number
  startTime?: string
  threadCount?: number
  architecture?: string
  description?: string
  publisher?: string
  signatureStatus: string
  accessStatus: string
  firstSeen: string
  lastSeen: string
  observationCount: number
  active: boolean
}

export type ProcessFilter = 'all' | 'user' | 'system' | 'high-cpu' | 'high-memory' | 'no-publisher' | 'restricted'

export interface ConnectionInfo {
  key: string
  protocol: 'tcp' | 'udp'
  ipVersion: 'ipv4' | 'ipv6'
  localAddress: string
  localPort: number
  remoteAddress?: string
  remotePort?: number
  state?: string
  pid?: number
  processName?: string
  executablePath?: string
  firstSeen: string
  lastSeen: string
  observationCount: number
  active: boolean
}

export type ConnectionFilter = 'all' | 'tcp' | 'udp' | 'ipv4' | 'ipv6' | 'established' | 'listening' | 'other'

export interface ServiceInfo {
  key: string
  serviceName: string
  displayName: string
  status: string
  startupType: string
  binaryPath?: string
  account?: string
  pid?: number
  firstSeen: string
  lastSeen: string
  observationCount: number
  active: boolean
}

export interface TelemetryEvent {
  id: string
  eventType: string
  subjectType: string
  subjectKey: string
  message: string
  occurredAt: string
}

export interface LiveTelemetrySnapshot {
  collectedAt: string
  processes: ProcessInfo[]
  connections: ConnectionInfo[]
  services: ServiceInfo[]
  events: TelemetryEvent[]
  collectors: Array<{
    id: string
    status: 'active' | 'partial'
    detail: string
    collectedAt: string
  }>
  issues: TelemetryIssue[]
}

export type ServiceFilter = 'all' | 'running' | 'stopped' | 'automatic' | 'manual' | 'disabled'

export interface CollectorHealth {
  key: CollectorKey
  label: string
  state: CollectorState
  issue?: string
  collectedAt?: string
}
