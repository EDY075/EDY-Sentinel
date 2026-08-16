export type CollectorKey = 'system' | 'processes' | 'network' | 'services'
export type CollectorState = 'healthy' | 'degraded' | 'failed' | 'paused' | 'loading'

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
  coreEquivalentCpuPercent: number | null
  memoryBytes: number
  startTime?: string
  threadCount?: number
  architecture?: string
  description?: string
  company?: string
  signatureStatus: string
  signer?: string
  executableFileSize?: number
  executableModifiedAt?: string
  accessStatus: string
  firstSeen: string
  lastSeen: string
  observationCount: number
  active: boolean
}

export type ProcessFilter = 'all' | 'user' | 'system' | 'high-cpu' | 'high-memory' | 'no-company' | 'restricted'

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
  associationStatus: 'associated' | 'recently_exited' | 'unresolved' | 'system_kernel' | 'not_applicable'
  processLastSeen?: string
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
  eventId: string
  eventType: string
  entityType: string
  entityKey: string
  timestamp: string
  collector: string
  factualPayload: Record<string, unknown>
  schemaVersion: number
  message: string
}

export interface CollectorTelemetry {
  id: string
  status: 'healthy' | 'degraded' | 'failed'
  detail: string
  lastSuccess?: string
  lastAttempt: string
  durationMs: number
  observationCount: number
  restrictedCount: number
  errorCode?: string
  errorMessage?: string
}

export interface LiveTelemetrySnapshot {
  collectedAt: string
  processes: ProcessInfo[]
  connections: ConnectionInfo[]
  services: ServiceInfo[]
  events: TelemetryEvent[]
  collectors: CollectorTelemetry[]
  issues: TelemetryIssue[]
}

export type ServiceFilter = 'all' | 'running' | 'stopped' | 'automatic' | 'manual' | 'disabled'

export interface CollectorHealth {
  key: CollectorKey
  label: string
  state: CollectorState
  issue?: string
  lastSuccess?: string
  lastAttempt?: string
  durationMs?: number
  observationCount: number
  restrictedCount: number
}
