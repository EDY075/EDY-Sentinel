use crate::models::{CollectionIssue, ProcessRecord};
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::SystemTime,
};
use sysinfo::{ProcessesToUpdate, System, Users};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, INVALID_HANDLE_VALUE, TRUST_E_NOSIGNATURE, TRUST_E_PROVIDER_UNKNOWN,
        TRUST_E_SUBJECT_FORM_UNKNOWN,
    },
    Security::WinTrust::{
        WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0,
        WINTRUST_FILE_INFO, WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_FILE, WTD_REVOKE_NONE,
        WTD_STATEACTION_IGNORE, WTD_UI_NONE,
    },
    Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
        },
        SystemInformation::{IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_I386},
        Threading::{IsWow64Process2, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};

#[derive(Clone)]
struct ExecutableMetadata {
    modified: Option<SystemTime>,
    description: Option<String>,
    publisher: Option<String>,
    signature_status: String,
}

impl Default for ExecutableMetadata {
    fn default() -> Self {
        Self {
            modified: None,
            description: None,
            publisher: None,
            signature_status: "unavailable".into(),
        }
    }
}

static METADATA_CACHE: OnceLock<Mutex<HashMap<PathBuf, ExecutableMetadata>>> = OnceLock::new();

pub struct ProcessCollector {
    system: System,
    users: Users,
    architectures: HashMap<String, Option<String>>,
    warmed: bool,
}

impl Default for ProcessCollector {
    fn default() -> Self {
        Self {
            system: System::new_all(),
            users: Users::new_with_refreshed_list(),
            architectures: HashMap::new(),
            warmed: false,
        }
    }
}

impl ProcessCollector {
    pub fn collect(&mut self) -> (Vec<ProcessRecord>, Vec<CollectionIssue>) {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        let thread_counts = collect_thread_counts();
        let now = Utc::now().to_rfc3339();
        let mut restricted = 0usize;
        let mut enrichment_budget = 24usize;

        let mut records: Vec<_> = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let pid_value = pid.as_u32();
                let executable = process.exe().map(Path::to_path_buf);
                let metadata = executable
                    .as_deref()
                    .map(|path| cached_metadata(path, &mut enrichment_budget))
                    .unwrap_or_default();
                let access_restricted = executable.is_none() && process.cmd().is_empty();
                if access_restricted {
                    restricted += 1;
                }
                let start_time = DateTime::from_timestamp(process.start_time() as i64, 0)
                    .map(|value| value.to_rfc3339());
                let key = format!("{}:{}", pid_value, process.start_time());
                ProcessRecord {
                    key,
                    name: process.name().to_string_lossy().into_owned(),
                    pid: pid_value,
                    parent_pid: process.parent().map(|value| value.as_u32()),
                    user: process
                        .user_id()
                        .and_then(|id| self.users.get_user_by_id(id))
                        .map(|user| user.name().to_string()),
                    executable_path: executable
                        .as_deref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    command_line: (!process.cmd().is_empty()).then(|| {
                        process
                            .cmd()
                            .iter()
                            .map(|part| part.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(" ")
                    }),
                    cpu_percent: self.warmed.then(|| process.cpu_usage()),
                    memory_bytes: process.memory(),
                    start_time,
                    thread_count: thread_counts.get(&pid_value).copied(),
                    architecture: self
                        .architectures
                        .entry(format!("{}:{}", pid_value, process.start_time()))
                        .or_insert_with(|| process_architecture(pid_value))
                        .clone(),
                    description: metadata.description,
                    publisher: metadata.publisher,
                    signature_status: metadata.signature_status,
                    access_status: if access_restricted {
                        "restricted".into()
                    } else if executable.is_none() || process.user_id().is_none() {
                        "partial".into()
                    } else {
                        "available".into()
                    },
                    first_seen: now.clone(),
                    last_seen: now.clone(),
                    observation_count: 1,
                    active: true,
                }
            })
            .collect();
        records.sort_by(|left, right| left.name.cmp(&right.name).then(left.pid.cmp(&right.pid)));

        let issues = if restricted == 0 {
            Vec::new()
        } else {
            vec![CollectionIssue {
            component: "processes".into(),
            message: format!(
                "Windows restricted executable details for {restricted} process(es); basic identity remains available"
            ),
        }]
        };
        self.architectures
            .retain(|key, _| records.iter().any(|record| &record.key == key));
        self.warmed = true;
        (records, issues)
    }
}

fn collect_thread_counts() -> HashMap<u32, u32> {
    let mut counts = HashMap::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return counts;
        }
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        if Thread32First(snapshot, &mut entry) != 0 {
            loop {
                *counts.entry(entry.th32OwnerProcessID).or_insert(0) += 1;
                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
    counts
}

fn process_architecture(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut process_machine = 0u16;
        let mut native_machine = 0u16;
        let ok = IsWow64Process2(handle, &mut process_machine, &mut native_machine) != 0;
        CloseHandle(handle);
        if !ok {
            return None;
        }
        Some(
            match process_machine {
                IMAGE_FILE_MACHINE_I386 => "x86",
                0 if native_machine == IMAGE_FILE_MACHINE_AMD64 => "x64",
                0 => "native",
                _ => "other",
            }
            .into(),
        )
    }
}

fn cached_metadata(path: &Path, enrichment_budget: &mut usize) -> ExecutableMetadata {
    let modified = path.metadata().ok().and_then(|value| value.modified().ok());
    let cache = METADATA_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(cache) = cache.lock() {
        if let Some(value) = cache.get(path) {
            if value.modified == modified {
                return value.clone();
            }
        }
    }
    if *enrichment_budget == 0 {
        return ExecutableMetadata::default();
    }
    *enrichment_budget -= 1;
    let (description, publisher) = read_version_metadata(path);
    let value = ExecutableMetadata {
        modified,
        description,
        publisher,
        signature_status: verify_signature(path),
    };
    if let Ok(mut cache) = cache.lock() {
        cache.insert(path.to_path_buf(), value.clone());
    }
    value
}

fn verify_signature(path: &Path) -> String {
    let path_wide = wide(path.as_os_str());
    unsafe {
        let mut file = WINTRUST_FILE_INFO {
            cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
            pcwszFilePath: path_wide.as_ptr(),
            hFile: std::ptr::null_mut(),
            pgKnownSubject: std::ptr::null_mut(),
        };
        let mut trust = WINTRUST_DATA {
            cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
            pPolicyCallbackData: std::ptr::null_mut(),
            pSIPClientData: std::ptr::null_mut(),
            dwUIChoice: WTD_UI_NONE,
            fdwRevocationChecks: WTD_REVOKE_NONE,
            dwUnionChoice: WTD_CHOICE_FILE,
            Anonymous: WINTRUST_DATA_0 { pFile: &mut file },
            dwStateAction: WTD_STATEACTION_IGNORE,
            hWVTStateData: std::ptr::null_mut(),
            pwszURLReference: std::ptr::null_mut(),
            dwProvFlags: WTD_CACHE_ONLY_URL_RETRIEVAL,
            dwUIContext: 0,
            pSignatureSettings: std::ptr::null_mut(),
        };
        let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        match WinVerifyTrust(
            std::ptr::null_mut(),
            &mut action,
            (&mut trust as *mut WINTRUST_DATA).cast(),
        ) {
            0 => "valid",
            TRUST_E_NOSIGNATURE | TRUST_E_PROVIDER_UNKNOWN | TRUST_E_SUBJECT_FORM_UNKNOWN => {
                "unsigned"
            }
            _ => "verification_failed",
        }
        .into()
    }
}

fn read_version_metadata(path: &Path) -> (Option<String>, Option<String>) {
    let wide = wide(path.as_os_str());
    unsafe {
        let mut ignored = 0u32;
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), &mut ignored);
        if size == 0 {
            return (None, None);
        }
        let mut data = vec![0u8; size as usize];
        if GetFileVersionInfoW(wide.as_ptr(), 0, size, data.as_mut_ptr().cast()) == 0 {
            return (None, None);
        }
        let translation = query_translation(&data).unwrap_or((0x0409, 0x04b0));
        (
            query_version_string(&data, translation, "FileDescription"),
            query_version_string(&data, translation, "CompanyName"),
        )
    }
}

unsafe fn query_translation(data: &[u8]) -> Option<(u16, u16)> {
    let key = wide(OsStr::new("\\VarFileInfo\\Translation"));
    let mut pointer = std::ptr::null_mut();
    let mut length = 0u32;
    if VerQueryValueW(
        data.as_ptr().cast(),
        key.as_ptr(),
        &mut pointer,
        &mut length,
    ) == 0
        || length < 4
    {
        return None;
    }
    let values = std::slice::from_raw_parts(pointer.cast::<u16>(), 2);
    Some((values[0], values[1]))
}

unsafe fn query_version_string(data: &[u8], translation: (u16, u16), name: &str) -> Option<String> {
    let query = format!(
        "\\StringFileInfo\\{:04x}{:04x}\\{}",
        translation.0, translation.1, name
    );
    let key = wide(OsStr::new(&query));
    let mut pointer = std::ptr::null_mut();
    let mut length = 0u32;
    if VerQueryValueW(
        data.as_ptr().cast(),
        key.as_ptr(),
        &mut pointer,
        &mut length,
    ) == 0
        || length == 0
        || pointer.is_null()
    {
        return None;
    }
    let base = data.as_ptr() as usize;
    let pointer_address = pointer as usize;
    let available_units = base
        .checked_add(data.len())
        .and_then(|end| end.checked_sub(pointer_address))
        .map(|bytes| bytes / std::mem::size_of::<u16>())?;
    let value = std::slice::from_raw_parts(
        pointer.cast::<u16>(),
        (length as usize).min(available_units),
    );
    let terminator = value
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(value.len());
    let text = String::from_utf16_lossy(&value[..terminator])
        .trim()
        .to_string();
    (!text.is_empty()).then_some(text)
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}
