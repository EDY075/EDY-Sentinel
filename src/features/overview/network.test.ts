import { describe, expect, it } from 'vitest'
import type { SystemOverview } from '../../types/system'
import { summarizePrimaryRoute } from './network'

describe('network overview semantics', () => {
  it('labels the Windows-selected route independently from physical adapter type', () => {
    const network = {
      primaryInterface: 'Radmin VPN',
      primaryInterfaceType: 'VPN',
      primaryIpv4: '26.1.2.3',
      primaryGateway: '26.0.0.1',
      primaryRouteMetric: 5,
      gateways: ['26.0.0.1'],
      dnsServers: ['1.1.1.1'],
      interfaces: [{ name: 'vpn', friendlyName: 'Radmin VPN', description: 'Virtual adapter', interfaceType: 'VPN', classificationSource: 'Windows interface type + documented heuristic', operationalStatus: 'Up', ipv4Metric: 5, primaryRoute: true, ipv4: ['26.1.2.3'], ipv6: [], gateways: ['26.0.0.1'], dnsServers: ['1.1.1.1'] }],
    } satisfies SystemOverview['network']
    expect(summarizePrimaryRoute(network)).toMatchObject({ label: 'Primary route', type: 'VPN', localIp: '26.1.2.3' })
  })
})
