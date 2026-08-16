use crate::models::{CollectionIssue, ConnectionRecord, ProcessRecord};
use chrono::Utc;
use std::{
    collections::HashMap,
    mem::size_of,
    net::{Ipv4Addr, Ipv6Addr},
};
use windows_sys::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR},
    NetworkManagement::IpHelper::{
        GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
        MIB_UDP6ROW_OWNER_PID, MIB_UDPROW_OWNER_PID, TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
    },
    Networking::WinSock::{AF_INET, AF_INET6},
};

pub fn collect(processes: &[ProcessRecord]) -> (Vec<ConnectionRecord>, Vec<CollectionIssue>) {
    let now = Utc::now().to_rfc3339();
    let mut records = Vec::new();
    let mut issues = Vec::new();

    for (label, component, result) in [
        ("TCP IPv4", "connections-tcp-ipv4", tcp_v4(&now)),
        ("TCP IPv6", "connections-tcp-ipv6", tcp_v6(&now)),
        ("UDP IPv4", "connections-udp-ipv4", udp_v4(&now)),
        ("UDP IPv6", "connections-udp-ipv6", udp_v6(&now)),
    ] {
        match result {
            Ok(mut values) => records.append(&mut values),
            Err(message) => issues.push(CollectionIssue {
                component: component.into(),
                message: format!("{label} table is unavailable: {message}"),
            }),
        }
    }

    correlate_processes(&mut records, processes);
    records.sort_by(|left, right| {
        left.protocol
            .cmp(&right.protocol)
            .then(left.local_address.cmp(&right.local_address))
            .then(left.local_port.cmp(&right.local_port))
    });
    (records, issues)
}

pub(crate) fn correlate_processes(
    connections: &mut [ConnectionRecord],
    processes: &[ProcessRecord],
) {
    let by_pid: HashMap<u32, &ProcessRecord> = processes
        .iter()
        .map(|process| (process.pid, process))
        .collect();
    for connection in connections {
        if let Some(process) = connection.pid.and_then(|pid| by_pid.get(&pid)) {
            connection.process_name = Some(process.name.clone());
            connection.executable_path = process.executable_path.clone();
        }
    }
}

fn tcp_v4(now: &str) -> Result<Vec<ConnectionRecord>, String> {
    let data = tcp_table(AF_INET as u32)?;
    parse_rows::<MIB_TCPROW_OWNER_PID>(&data, |row| {
        let local_address = Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()).to_string();
        let local_port = port(row.dwLocalPort);
        let is_listener = row.dwState == 2;
        connection(
            "tcp",
            "ipv4",
            local_address,
            local_port,
            (!is_listener).then(|| Ipv4Addr::from(row.dwRemoteAddr.to_ne_bytes()).to_string()),
            (!is_listener).then(|| port(row.dwRemotePort)),
            Some(tcp_state(row.dwState).into()),
            (row.dwOwningPid != 0).then_some(row.dwOwningPid),
            now,
        )
    })
}

fn tcp_v6(now: &str) -> Result<Vec<ConnectionRecord>, String> {
    let data = tcp_table(AF_INET6 as u32)?;
    parse_rows::<MIB_TCP6ROW_OWNER_PID>(&data, |row| {
        let local_address = ipv6(row.ucLocalAddr, row.dwLocalScopeId);
        let local_port = port(row.dwLocalPort);
        let is_listener = row.dwState == 2;
        connection(
            "tcp",
            "ipv6",
            local_address,
            local_port,
            (!is_listener).then(|| ipv6(row.ucRemoteAddr, row.dwRemoteScopeId)),
            (!is_listener).then(|| port(row.dwRemotePort)),
            Some(tcp_state(row.dwState).into()),
            (row.dwOwningPid != 0).then_some(row.dwOwningPid),
            now,
        )
    })
}

fn udp_v4(now: &str) -> Result<Vec<ConnectionRecord>, String> {
    let data = udp_table(AF_INET as u32)?;
    parse_rows::<MIB_UDPROW_OWNER_PID>(&data, |row| {
        connection(
            "udp",
            "ipv4",
            Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes()).to_string(),
            port(row.dwLocalPort),
            None,
            None,
            None,
            (row.dwOwningPid != 0).then_some(row.dwOwningPid),
            now,
        )
    })
}

fn udp_v6(now: &str) -> Result<Vec<ConnectionRecord>, String> {
    let data = udp_table(AF_INET6 as u32)?;
    parse_rows::<MIB_UDP6ROW_OWNER_PID>(&data, |row| {
        connection(
            "udp",
            "ipv6",
            ipv6(row.ucLocalAddr, row.dwLocalScopeId),
            port(row.dwLocalPort),
            None,
            None,
            None,
            (row.dwOwningPid != 0).then_some(row.dwOwningPid),
            now,
        )
    })
}

fn tcp_table(family: u32) -> Result<Vec<u64>, String> {
    table_buffer(|pointer, size| unsafe {
        GetExtendedTcpTable(pointer, size, 1, family, TCP_TABLE_OWNER_PID_ALL, 0)
    })
}

fn udp_table(family: u32) -> Result<Vec<u64>, String> {
    table_buffer(|pointer, size| unsafe {
        GetExtendedUdpTable(pointer, size, 1, family, UDP_TABLE_OWNER_PID, 0)
    })
}

fn table_buffer(call: impl Fn(*mut std::ffi::c_void, &mut u32) -> u32) -> Result<Vec<u64>, String> {
    let mut byte_size = 0u32;
    let first = call(std::ptr::null_mut(), &mut byte_size);
    if first != ERROR_INSUFFICIENT_BUFFER && first != NO_ERROR {
        return Err(format!("Windows error {first}"));
    }
    if byte_size < 4 {
        return Ok(vec![0]);
    }
    let mut buffer = vec![0u64; (byte_size as usize).div_ceil(size_of::<u64>())];
    let result = call(buffer.as_mut_ptr().cast(), &mut byte_size);
    if result == NO_ERROR {
        Ok(buffer)
    } else {
        Err(format!("Windows error {result}"))
    }
}

fn parse_rows<T: Copy>(
    data: &[u64],
    mut map: impl FnMut(T) -> ConnectionRecord,
) -> Result<Vec<ConnectionRecord>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let pointer = data.as_ptr().cast::<u8>();
    let count = unsafe { std::ptr::read_unaligned(pointer.cast::<u32>()) } as usize;
    let available = std::mem::size_of_val(data);
    let required = 4usize.saturating_add(count.saturating_mul(size_of::<T>()));
    if required > available {
        return Err("Windows returned a truncated table".into());
    }
    let mut rows = Vec::with_capacity(count);
    for index in 0..count {
        let offset = 4 + index * size_of::<T>();
        let row = unsafe { std::ptr::read_unaligned(pointer.add(offset).cast::<T>()) };
        rows.push(map(row));
    }
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
fn connection(
    protocol: &str,
    ip_version: &str,
    local_address: String,
    local_port: u16,
    remote_address: Option<String>,
    remote_port: Option<u16>,
    state: Option<String>,
    pid: Option<u32>,
    now: &str,
) -> ConnectionRecord {
    let key = format!(
        "{protocol}|{ip_version}|{local_address}|{local_port}|{}|{}|{}",
        remote_address.as_deref().unwrap_or("-"),
        remote_port.map_or_else(|| "-".into(), |value| value.to_string()),
        pid.map_or_else(|| "-".into(), |value| value.to_string())
    );
    ConnectionRecord {
        key,
        protocol: protocol.into(),
        ip_version: ip_version.into(),
        local_address,
        local_port,
        remote_address,
        remote_port,
        state,
        pid,
        process_name: None,
        executable_path: None,
        first_seen: now.into(),
        last_seen: now.into(),
        observation_count: 1,
        active: true,
    }
}

fn port(value: u32) -> u16 {
    u16::from_be(value as u16)
}

fn ipv6(address: [u8; 16], scope_id: u32) -> String {
    let address = Ipv6Addr::from(address).to_string();
    if scope_id == 0 {
        address
    } else {
        format!("{address}%{scope_id}")
    }
}

fn tcp_state(value: u32) -> &'static str {
    match value {
        1 => "closed",
        2 => "listening",
        3 => "syn_sent",
        4 => "syn_received",
        5 => "established",
        6 => "fin_wait_1",
        7 => "fin_wait_2",
        8 => "close_wait",
        9 => "closing",
        10 => "last_ack",
        11 => "time_wait",
        12 => "delete_tcb",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(pid: u32) -> ProcessRecord {
        ProcessRecord {
            key: format!("{pid}:1"),
            name: "sample.exe".into(),
            pid,
            parent_pid: None,
            user: None,
            executable_path: Some("C:\\sample.exe".into()),
            command_line: None,
            cpu_percent: Some(0.0),
            memory_bytes: 0,
            start_time: None,
            thread_count: None,
            architecture: None,
            description: None,
            publisher: None,
            signature_status: "not_checked".into(),
            access_status: "partial".into(),
            first_seen: "now".into(),
            last_seen: "now".into(),
            observation_count: 1,
            active: true,
        }
    }

    #[test]
    fn pid_correlation_adds_only_observed_process_details() {
        let mut values = vec![connection(
            "tcp",
            "ipv4",
            "127.0.0.1".into(),
            443,
            None,
            None,
            None,
            Some(42),
            "now",
        )];
        correlate_processes(&mut values, &[process(42)]);
        assert_eq!(values[0].process_name.as_deref(), Some("sample.exe"));
        assert_eq!(values[0].executable_path.as_deref(), Some("C:\\sample.exe"));
    }

    #[test]
    fn windows_port_encoding_is_parsed_as_network_byte_order() {
        assert_eq!(port(0x5000), 80);
        assert_eq!(tcp_state(5), "established");
    }
}
