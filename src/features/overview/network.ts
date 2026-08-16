import type { SystemOverview } from '../../types/system'

export function summarizePrimaryRoute(network: SystemOverview['network']) {
  return {
    label: 'Primary route',
    name: network.primaryInterface ?? 'Unavailable',
    type: network.primaryInterfaceType ?? 'Adapter type unavailable',
    localIp: network.primaryIpv4 ?? 'Unavailable',
    gateway: network.primaryGateway ?? network.gateways[0] ?? 'Unavailable',
    additionalInterfaces: Math.max(0, network.interfaces.length - 1),
  }
}
