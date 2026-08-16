use crate::models::*;
use chrono::Utc;
use serde::Deserialize;
use sysinfo::{Disks, System};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct VideoController {
    name: Option<String>,
    adapter_ram: Option<u64>,
    driver_version: Option<String>,
}

pub fn collect(
    network_result: (NetworkInfo, Option<CollectionIssue>),
) -> Result<SystemOverview, String> {
    let mut system = System::new_all();
    system.refresh_all();
    let mut issues = Vec::new();
    if let Some(issue) = network_result.1 {
        issues.push(issue);
    }

    let current_version = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .ok();
    let product_name: Option<String> = current_version
        .as_ref()
        .and_then(|key| key.get_value("ProductName").ok());
    let edition: Option<String> = current_version
        .as_ref()
        .and_then(|key| key.get_value("EditionID").ok());
    let display_version: Option<String> = current_version
        .as_ref()
        .and_then(|key| key.get_value("DisplayVersion").ok());
    let build: Option<String> = current_version
        .as_ref()
        .and_then(|key| key.get_value("CurrentBuildNumber").ok());
    let windows_name = match (&product_name, &build) {
        (Some(name), Some(build_number)) if name.contains("Windows 10") => build_number
            .parse::<u32>()
            .ok()
            .filter(|number| *number >= 22000)
            .map(|_| name.replace("Windows 10", "Windows 11"))
            .unwrap_or_else(|| name.clone()),
        (Some(name), _) => name.clone(),
        _ => System::long_os_version().unwrap_or_else(|| "Windows".into()),
    };

    let cpus = system.cpus();
    let cpu = CpuInfo {
        model: cpus
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Unavailable".into()),
        logical_cores: cpus.len(),
        physical_cores: System::physical_core_count(),
        frequency_mhz: cpus.first().map(|cpu| cpu.frequency()).unwrap_or_default(),
    };
    if cpu.model == "Unavailable" || cpu.logical_cores == 0 {
        issues.push(CollectionIssue {
            component: "cpu".into(),
            message: "Processor identity or core count is unavailable".into(),
        });
    }

    let gpus = match collect_gpus() {
        Ok(items) if items.is_empty() => {
            issues.push(CollectionIssue {
                component: "gpu".into(),
                message: "No display adapter was returned by Windows Management Instrumentation"
                    .into(),
            });
            Vec::new()
        }
        Ok(items) => items,
        Err(_) => {
            issues.push(CollectionIssue {
                component: "gpu".into(),
                message: "GPU details are temporarily unavailable from Windows Management Instrumentation".into(),
            });
            Vec::new()
        }
    };

    let disks: Vec<DiskInfo> = Disks::new_with_refreshed_list()
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
            removable: disk.is_removable(),
        })
        .collect();
    if disks.is_empty() {
        issues.push(CollectionIssue {
            component: "storage".into(),
            message: "No local volumes were returned by the Windows storage APIs".into(),
        });
    }

    let host = HostInfo {
        hostname: System::host_name().unwrap_or_else(|| "Unavailable".into()),
        username: std::env::var("USERNAME").unwrap_or_else(|_| "Unavailable".into()),
        architecture: std::env::consts::ARCH.into(),
        uptime_seconds: System::uptime(),
    };
    if host.hostname == "Unavailable" || host.username == "Unavailable" {
        issues.push(CollectionIssue {
            component: "host".into(),
            message: "Hostname or signed-in user is unavailable".into(),
        });
    }

    let memory = MemoryInfo {
        total_bytes: system.total_memory(),
        used_bytes: system.used_memory(),
    };
    if memory.total_bytes == 0 || memory.used_bytes > memory.total_bytes {
        issues.push(CollectionIssue {
            component: "memory".into(),
            message: "Windows returned incomplete memory capacity data".into(),
        });
    }

    let network = network_result.0;
    if network.primary_interface.is_none() || network.primary_ipv4.is_none() {
        issues.push(CollectionIssue {
            component: "network".into(),
            message: "No primary interface with an IPv4 address was detected".into(),
        });
    }
    if network.gateways.is_empty() || network.dns_servers.is_empty() {
        issues.push(CollectionIssue {
            component: "network-route".into(),
            message: "Default gateway or DNS resolver details are incomplete".into(),
        });
    }

    Ok(SystemOverview {
        collected_at: Utc::now().to_rfc3339(),
        source: "Windows Registry, WMI, IP Helper and native system APIs".into(),
        host,
        operating_system: OperatingSystemInfo {
            name: windows_name,
            edition,
            display_version,
            build,
        },
        cpu,
        gpus,
        memory,
        disks,
        network,
        issues,
    })
}

fn collect_gpus() -> Result<Vec<GpuInfo>, String> {
    let connection = wmi::WMIConnection::new().map_err(|_| "WMI unavailable".to_string())?;
    let controllers: Vec<VideoController> = connection
        .raw_query("SELECT Name, AdapterRAM, DriverVersion FROM Win32_VideoController")
        .map_err(|_| "GPU query unavailable".to_string())?;
    Ok(controllers
        .into_iter()
        .filter_map(|gpu| {
            gpu.name.map(|name| GpuInfo {
                name,
                adapter_ram_bytes: gpu.adapter_ram,
                driver_version: gpu.driver_version,
            })
        })
        .collect())
}
