import type { SystemOverview } from '../../types/system'

export function summarizePrimaryRoute(network: SystemOverview['network'], labels = { primaryRoute: 'Primary route', unavailable: 'Unavailable', adapterTypeUnavailable: 'Adapter type unavailable' }) {
  return {
    label: labels.primaryRoute,
    name: network.primaryInterface ?? labels.unavailable,
    type: network.primaryInterfaceType ?? labels.adapterTypeUnavailable,
    localIp: network.primaryIpv4 ?? labels.unavailable,
    gateway: network.primaryGateway ?? network.gateways[0] ?? labels.unavailable,
    additionalInterfaces: Math.max(0, network.interfaces.length - 1),
  }
}
