use crate::models::{CollectionIssue, NetworkInfo, NetworkInterface};
use ipconfig::{IfType, OperStatus};
use std::net::{IpAddr, Ipv4Addr};
use windows_sys::Win32::{
    NetworkManagement::IpHelper::{GetBestRoute2, MIB_IPFORWARD_ROW2},
    Networking::WinSock::{AF_INET, IN_ADDR, IN_ADDR_0, SOCKADDR_IN, SOCKADDR_INET},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PrimaryRoute {
    source: String,
    gateway: Option<String>,
    interface_index: u32,
    metric: u32,
}

pub fn collect() -> (NetworkInfo, Option<CollectionIssue>) {
    match ipconfig::get_adapters() {
        Ok(adapters) => {
            let route = best_ipv4_route();
            let primary_position = route
                .as_ref()
                .and_then(|best| {
                    adapters.iter().position(|adapter| {
                        adapter
                            .ip_addresses()
                            .iter()
                            .any(|address| address.to_string() == best.source)
                            || adapter.ipv6_if_index() == best.interface_index
                    })
                })
                .or_else(|| {
                    adapters
                        .iter()
                        .enumerate()
                        .filter(|(_, adapter)| {
                            adapter.oper_status() == OperStatus::IfOperStatusUp
                                && adapter.ip_addresses().iter().any(
                                    |address| matches!(address, IpAddr::V4(value) if !value.is_loopback()),
                                )
                                && !adapter.gateways().is_empty()
                        })
                        .min_by_key(|(_, adapter)| adapter.ipv4_metric())
                        .map(|(index, _)| index)
                });

            let mut result = NetworkInfo::default();
            for (position, adapter) in adapters.into_iter().enumerate() {
                let ipv4: Vec<String> = adapter
                    .ip_addresses()
                    .iter()
                    .filter(|ip| matches!(ip, IpAddr::V4(_)))
                    .map(ToString::to_string)
                    .collect();
                let ipv6: Vec<String> = adapter
                    .ip_addresses()
                    .iter()
                    .filter(|ip| matches!(ip, IpAddr::V6(_)))
                    .map(ToString::to_string)
                    .collect();
                let gateways: Vec<String> =
                    adapter.gateways().iter().map(ToString::to_string).collect();
                let dns_servers: Vec<String> = adapter
                    .dns_servers()
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                let (interface_type, classification_source) = classify_interface(
                    adapter.if_type(),
                    adapter.friendly_name(),
                    adapter.description(),
                );
                let primary_route = Some(position) == primary_position;

                if primary_route {
                    result.primary_interface = Some(adapter.friendly_name().to_string());
                    result.primary_interface_type = Some(interface_type.clone());
                    result.primary_ipv4 = route
                        .as_ref()
                        .map(|value| value.source.clone())
                        .or_else(|| ipv4.iter().find(|ip| !ip.starts_with("127.")).cloned());
                    result.primary_gateway = route
                        .as_ref()
                        .and_then(|value| value.gateway.clone())
                        .or_else(|| gateways.first().cloned());
                    result.primary_route_metric = route
                        .as_ref()
                        .map(|value| value.metric.saturating_add(adapter.ipv4_metric()))
                        .or_else(|| Some(adapter.ipv4_metric()));
                    result.gateways = gateways.clone();
                    result.dns_servers = dns_servers.clone();
                }

                if !ipv4.is_empty() || !ipv6.is_empty() {
                    result.interfaces.push(NetworkInterface {
                        name: adapter.adapter_name().to_string(),
                        friendly_name: adapter.friendly_name().to_string(),
                        description: adapter.description().to_string(),
                        interface_type,
                        classification_source,
                        operational_status: oper_status(adapter.oper_status()).into(),
                        ipv4_metric: adapter.ipv4_metric(),
                        primary_route,
                        ipv4,
                        ipv6,
                        gateways,
                        dns_servers,
                    });
                }
            }
            (result, None)
        }
        Err(_) => (
            NetworkInfo::default(),
            Some(CollectionIssue {
                component: "network".into(),
                message: "Windows network adapter data is temporarily unavailable".into(),
            }),
        ),
    }
}

fn classify_interface(if_type: IfType, friendly_name: &str, description: &str) -> (String, String) {
    let combined = format!("{friendly_name} {description}").to_lowercase();
    let heuristic = if [
        "vpn",
        "radmin",
        "wireguard",
        "openvpn",
        "zerotier",
        "tailscale",
    ]
    .iter()
    .any(|marker| combined.contains(marker))
    {
        Some("VPN")
    } else if [
        "virtual",
        "hyper-v",
        "vmware",
        "virtualbox",
        "vbox",
        "tap",
        "npcap",
    ]
    .iter()
    .any(|marker| combined.contains(marker))
    {
        Some("Virtual")
    } else {
        None
    };
    if let Some(value) = heuristic {
        return (
            value.into(),
            "Windows interface type + documented heuristic".into(),
        );
    }
    let value = match if_type {
        IfType::EthernetCsmacd => "Physical Ethernet",
        IfType::Ieee80211 => "Wi-Fi",
        IfType::Ppp => "VPN",
        IfType::SoftwareLoopback => "Loopback",
        IfType::Tunnel => "Tunnel",
        _ => "Other",
    };
    (value.into(), "Windows interface type".into())
}

fn oper_status(status: OperStatus) -> &'static str {
    match status {
        OperStatus::IfOperStatusUp => "Up",
        OperStatus::IfOperStatusDown => "Down",
        OperStatus::IfOperStatusDormant => "Dormant",
        OperStatus::IfOperStatusNotPresent => "Not present",
        OperStatus::IfOperStatusLowerLayerDown => "Lower layer down",
        OperStatus::IfOperStatusTesting => "Testing",
        _ => "Unknown",
    }
}

fn best_ipv4_route() -> Option<PrimaryRoute> {
    unsafe {
        let destination = SOCKADDR_INET {
            Ipv4: SOCKADDR_IN {
                sin_family: AF_INET as _,
                sin_port: 0,
                sin_addr: IN_ADDR {
                    S_un: IN_ADDR_0 {
                        S_addr: u32::from_ne_bytes([1, 1, 1, 1]),
                    },
                },
                sin_zero: [0; 8],
            },
        };
        let mut route = MIB_IPFORWARD_ROW2::default();
        let mut source = SOCKADDR_INET::default();
        if GetBestRoute2(
            std::ptr::null(),
            0,
            std::ptr::null(),
            &destination,
            0,
            &mut route,
            &mut source,
        ) != 0
        {
            return None;
        }
        let source_v4 = source.Ipv4;
        let next_hop = route.NextHop.Ipv4;
        let source = Ipv4Addr::from(source_v4.sin_addr.S_un.S_addr.to_ne_bytes()).to_string();
        let gateway_value = Ipv4Addr::from(next_hop.sin_addr.S_un.S_addr.to_ne_bytes());
        Some(PrimaryRoute {
            source,
            gateway: (!gateway_value.is_unspecified()).then(|| gateway_value.to_string()),
            interface_index: route.InterfaceIndex,
            metric: route.Metric,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_windows_interface_types_before_using_heuristics() {
        assert_eq!(
            classify_interface(IfType::Ieee80211, "Wi-Fi", "Intel").0,
            "Wi-Fi"
        );
        assert_eq!(
            classify_interface(IfType::EthernetCsmacd, "Radmin VPN", "Radmin adapter").0,
            "VPN"
        );
        assert_eq!(
            classify_interface(IfType::Tunnel, "IP Tunnel", "Windows tunnel").0,
            "Tunnel"
        );
    }
}
