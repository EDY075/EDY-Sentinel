use crate::models::{CollectionIssue, NetworkInfo, NetworkInterface};
use std::net::IpAddr;

pub fn collect() -> (NetworkInfo, Option<CollectionIssue>) {
    match ipconfig::get_adapters() {
        Ok(adapters) => {
            let mut result = NetworkInfo::default();
            for adapter in adapters {
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

                let is_primary = result.primary_interface.is_none()
                    && ipv4.iter().any(|ip| !ip.starts_with("127."))
                    && !gateways.is_empty();
                if is_primary {
                    result.primary_interface = Some(adapter.friendly_name().to_string());
                    result.primary_ipv4 = ipv4.first().cloned();
                    result.gateways = gateways.clone();
                    result.dns_servers = dns_servers.clone();
                }

                if !ipv4.is_empty() || !ipv6.is_empty() {
                    result.interfaces.push(NetworkInterface {
                        name: adapter.adapter_name().to_string(),
                        friendly_name: adapter.friendly_name().to_string(),
                        ipv4,
                        ipv6,
                        gateways,
                        dns_servers,
                    });
                }
            }

            if result.primary_interface.is_none() {
                if let Some(fallback) = result
                    .interfaces
                    .iter()
                    .find(|item| item.ipv4.iter().any(|ip| !ip.starts_with("127.")))
                {
                    result.primary_interface = Some(fallback.friendly_name.clone());
                    result.primary_ipv4 = fallback.ipv4.first().cloned();
                    result.gateways = fallback.gateways.clone();
                    result.dns_servers = fallback.dns_servers.clone();
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
