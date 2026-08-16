use crate::models::{CollectionIssue, ServiceRecord};
use chrono::Utc;
use std::{ffi::OsStr, mem::size_of, os::windows::ffi::OsStrExt};
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::Services::{
        CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, OpenServiceW,
        QueryServiceConfig2W, QueryServiceConfigW, ENUM_SERVICE_STATUS_PROCESSW,
        QUERY_SERVICE_CONFIGW, SC_ENUM_PROCESS_INFO, SC_MANAGER_CONNECT,
        SC_MANAGER_ENUMERATE_SERVICE, SERVICE_AUTO_START, SERVICE_BOOT_START,
        SERVICE_CONFIG_DELAYED_AUTO_START_INFO, SERVICE_CONTINUE_PENDING,
        SERVICE_DELAYED_AUTO_START_INFO, SERVICE_DEMAND_START, SERVICE_DISABLED, SERVICE_PAUSED,
        SERVICE_PAUSE_PENDING, SERVICE_QUERY_CONFIG, SERVICE_RUNNING, SERVICE_START_PENDING,
        SERVICE_STATE_ALL, SERVICE_STOPPED, SERVICE_STOP_PENDING, SERVICE_SYSTEM_START,
        SERVICE_WIN32,
    },
};

pub fn collect() -> (Vec<ServiceRecord>, Vec<CollectionIssue>) {
    match collect_inner() {
        Ok((records, restricted)) => {
            let issues = if restricted == 0 {
                Vec::new()
            } else {
                vec![CollectionIssue {
                    component: "services".into(),
                    message: format!(
                        "Windows restricted configuration details for {restricted} service(s); runtime state remains available"
                    ),
                }]
            };
            (records, issues)
        }
        Err(message) => (
            Vec::new(),
            vec![CollectionIssue {
                component: "services".into(),
                message,
            }],
        ),
    }
}

fn collect_inner() -> Result<(Vec<ServiceRecord>, usize), String> {
    unsafe {
        let manager = OpenSCManagerW(
            std::ptr::null(),
            std::ptr::null(),
            SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE,
        );
        if manager.is_null() {
            return Err(format!(
                "Windows Service Control Manager is unavailable (error {})",
                GetLastError()
            ));
        }

        let mut bytes_needed = 0u32;
        let mut returned = 0u32;
        let mut resume = 0u32;
        EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            std::ptr::null_mut(),
            0,
            &mut bytes_needed,
            &mut returned,
            &mut resume,
            std::ptr::null(),
        );
        if bytes_needed == 0 {
            let error = GetLastError();
            CloseServiceHandle(manager);
            return Err(format!("Service enumeration failed (error {error})"));
        }

        let mut buffer = vec![0u64; (bytes_needed as usize).div_ceil(size_of::<u64>())];
        resume = 0;
        let ok = EnumServicesStatusExW(
            manager,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            buffer.as_mut_ptr().cast(),
            bytes_needed,
            &mut bytes_needed,
            &mut returned,
            &mut resume,
            std::ptr::null(),
        );
        if ok == 0 {
            let error = GetLastError();
            CloseServiceHandle(manager);
            return Err(format!("Service enumeration failed (error {error})"));
        }

        let now = Utc::now().to_rfc3339();
        let rows = std::slice::from_raw_parts(
            buffer.as_ptr().cast::<ENUM_SERVICE_STATUS_PROCESSW>(),
            returned as usize,
        );
        let mut records = Vec::with_capacity(rows.len());
        let mut restricted = 0usize;
        for row in rows {
            let service_name = wide_pointer_to_string(row.lpServiceName);
            let display_name = wide_pointer_to_string(row.lpDisplayName);
            let config = query_config(manager, &service_name);
            if config.is_none() {
                restricted += 1;
            }
            let (startup_type, binary_path, account) =
                config.unwrap_or_else(|| ("Unavailable".into(), None, None));
            records.push(ServiceRecord {
                key: service_name.to_lowercase(),
                service_name,
                display_name,
                status: service_status(row.ServiceStatusProcess.dwCurrentState).into(),
                startup_type,
                binary_path,
                account,
                pid: (row.ServiceStatusProcess.dwProcessId != 0)
                    .then_some(row.ServiceStatusProcess.dwProcessId),
                first_seen: now.clone(),
                last_seen: now.clone(),
                observation_count: 1,
                active: true,
            });
        }
        CloseServiceHandle(manager);
        records.sort_by(|left, right| left.display_name.cmp(&right.display_name));
        Ok((records, restricted))
    }
}

unsafe fn query_config(
    manager: windows_sys::Win32::System::Services::SC_HANDLE,
    service_name: &str,
) -> Option<(String, Option<String>, Option<String>)> {
    let wide_name = wide(OsStr::new(service_name));
    let service = OpenServiceW(manager, wide_name.as_ptr(), SERVICE_QUERY_CONFIG);
    if service.is_null() {
        return None;
    }
    let mut needed = 0u32;
    QueryServiceConfigW(service, std::ptr::null_mut(), 0, &mut needed);
    if needed == 0 {
        CloseServiceHandle(service);
        return None;
    }
    let mut buffer = vec![0u64; (needed as usize).div_ceil(size_of::<u64>())];
    let config = buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
    if QueryServiceConfigW(service, config, needed, &mut needed) == 0 {
        CloseServiceHandle(service);
        return None;
    }
    let config_ref = &*config;
    let delayed = if config_ref.dwStartType == SERVICE_AUTO_START {
        let mut info = SERVICE_DELAYED_AUTO_START_INFO {
            fDelayedAutostart: 0,
        };
        let mut info_needed = 0u32;
        QueryServiceConfig2W(
            service,
            SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
            (&mut info as *mut SERVICE_DELAYED_AUTO_START_INFO).cast(),
            size_of::<SERVICE_DELAYED_AUTO_START_INFO>() as u32,
            &mut info_needed,
        ) != 0
            && info.fDelayedAutostart != 0
    } else {
        false
    };
    let startup = startup_type(config_ref.dwStartType, delayed).to_string();
    let binary = nullable_wide_pointer_to_string(config_ref.lpBinaryPathName);
    let account = nullable_wide_pointer_to_string(config_ref.lpServiceStartName);
    CloseServiceHandle(service);
    Some((startup, binary, account))
}

fn service_status(value: u32) -> &'static str {
    match value {
        SERVICE_STOPPED => "Stopped",
        SERVICE_START_PENDING => "Start Pending",
        SERVICE_STOP_PENDING => "Stop Pending",
        SERVICE_RUNNING => "Running",
        SERVICE_CONTINUE_PENDING => "Continue Pending",
        SERVICE_PAUSE_PENDING => "Pause Pending",
        SERVICE_PAUSED => "Paused",
        _ => "Unknown",
    }
}

fn startup_type(value: u32, delayed: bool) -> &'static str {
    match value {
        SERVICE_BOOT_START => "Boot",
        SERVICE_SYSTEM_START => "System",
        SERVICE_AUTO_START if delayed => "Automatic Delayed",
        SERVICE_AUTO_START => "Automatic",
        SERVICE_DEMAND_START => "Manual",
        SERVICE_DISABLED => "Disabled",
        _ => "Unknown",
    }
}

unsafe fn nullable_wide_pointer_to_string(pointer: *mut u16) -> Option<String> {
    (!pointer.is_null())
        .then(|| wide_pointer_to_string(pointer))
        .filter(|value| !value.is_empty())
}

unsafe fn wide_pointer_to_string(pointer: *mut u16) -> String {
    if pointer.is_null() {
        return String::new();
    }
    let mut length = 0usize;
    while *pointer.add(length) != 0 {
        length += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length))
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_states_are_factual_windows_states() {
        assert_eq!(service_status(SERVICE_RUNNING), "Running");
        assert_eq!(service_status(SERVICE_START_PENDING), "Start Pending");
        assert_eq!(startup_type(SERVICE_AUTO_START, true), "Automatic Delayed");
        assert_eq!(startup_type(SERVICE_DISABLED, false), "Disabled");
    }
}
