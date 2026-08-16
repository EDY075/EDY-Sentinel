export type ThemeName = 'sentinel-blue' | 'cyber-green' | 'terminal' | 'spectrum'

export interface SystemOverview {
  collectedAt: string
  source: string
  host: {
    hostname: string
    username: string
    architecture: string
    uptimeSeconds: number
  }
  operatingSystem: {
    name: string
    edition?: string
    displayVersion?: string
    build?: string
  }
  cpu: {
    model: string
    logicalCores: number
    physicalCores?: number
    frequencyMhz: number
  }
  gpus: Array<{
    name: string
    adapterRamBytes?: number
    driverVersion?: string
  }>
  memory: {
    totalBytes: number
    usedBytes: number
  }
  disks: Array<{
    name: string
    mountPoint: string
    fileSystem: string
    totalBytes: number
    availableBytes: number
    removable: boolean
  }>
  network: {
    primaryInterface?: string
    primaryInterfaceType?: string
    primaryIpv4?: string
    primaryGateway?: string
    primaryRouteMetric?: number
    gateways: string[]
    dnsServers: string[]
    interfaces: Array<{
      name: string
      friendlyName: string
      description: string
      interfaceType: string
      classificationSource: string
      operationalStatus: string
      ipv4Metric: number
      primaryRoute: boolean
      ipv4: string[]
      ipv6: string[]
      gateways: string[]
      dnsServers: string[]
    }>
  }
  collector: {
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
  issues: Array<{ component: string; message: string }>
}

export interface DatabaseStatus {
  schemaVersion: number
  pathKind: string
  writable: boolean
}
