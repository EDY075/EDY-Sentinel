use sha2::{Digest, Sha256};
use windows_sys::Win32::{
    Storage::FileSystem::{GetVolumeInformationW, GetVolumePathNameW},
    System::SystemInformation::GetSystemWindowsDirectoryW,
};
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

const MACHINE_GUID_KEY: &str = "SOFTWARE\\Microsoft\\Cryptography";
const MACHINE_GUID_VALUE: &str = "MachineGuid";
const INITIAL_WINDOWS_PATH_CAPACITY: usize = 260;
const MAX_WINDOWS_PATH_CAPACITY: usize = 32_768;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostIdentitySignals {
    pub(crate) machine_guid_available: bool,
    pub(crate) system_volume_serial_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostIdentity {
    pub(crate) host_id: String,
    pub(crate) signals: HostIdentitySignals,
}

pub(crate) fn current_host_identity() -> Result<HostIdentity, String> {
    collect_host_identity(&WindowsHostIdentitySource)
}

trait HostIdentitySource {
    fn machine_guid(&self) -> Option<String>;
    fn system_windows_directory(&self) -> Result<String, String>;
    fn volume_path_for(&self, path: &str) -> Result<String, String>;
    fn volume_serial(&self, volume_root: &str) -> Result<u32, String>;
}

struct WindowsHostIdentitySource;

impl HostIdentitySource for WindowsHostIdentitySource {
    fn machine_guid(&self) -> Option<String> {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(MACHINE_GUID_KEY)
            .and_then(|key| key.get_value::<String, _>(MACHINE_GUID_VALUE))
            .ok()
            .and_then(non_empty)
    }

    fn system_windows_directory(&self) -> Result<String, String> {
        let mut capacity = INITIAL_WINDOWS_PATH_CAPACITY;
        loop {
            let mut buffer = vec![0u16; capacity];
            let length = unsafe {
                GetSystemWindowsDirectoryW(buffer.as_mut_ptr(), buffer.len().try_into().unwrap())
            } as usize;
            if length == 0 {
                return Err("Windows system directory is unavailable".into());
            }
            if length < buffer.len() {
                buffer.truncate(length);
                return String::from_utf16(&buffer)
                    .map_err(|_| "Windows system directory is invalid".into());
            }
            capacity = length.saturating_add(1).min(MAX_WINDOWS_PATH_CAPACITY);
            if capacity <= buffer.len() {
                return Err("Windows system directory is too long".into());
            }
        }
    }

    fn volume_path_for(&self, path: &str) -> Result<String, String> {
        let path = wide(path);
        let mut buffer = vec![0u16; MAX_WINDOWS_PATH_CAPACITY];
        let success = unsafe {
            GetVolumePathNameW(
                path.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len().try_into().unwrap(),
            )
        };
        if success == 0 {
            return Err("Windows system volume path is unavailable".into());
        }
        decode_null_terminated(&buffer, "Windows system volume path is invalid")
            .map(ensure_trailing_separator)
    }

    fn volume_serial(&self, volume_root: &str) -> Result<u32, String> {
        let volume_root = wide(volume_root);
        let mut serial = 0u32;
        let success = unsafe {
            GetVolumeInformationW(
                volume_root.as_ptr(),
                std::ptr::null_mut(),
                0,
                &mut serial,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        };
        if success == 0 {
            Err("Windows system volume identity is unavailable".into())
        } else {
            Ok(serial)
        }
    }
}

fn collect_host_identity(source: &impl HostIdentitySource) -> Result<HostIdentity, String> {
    let machine_guid = source.machine_guid().and_then(non_empty);
    let volume_serial = source
        .system_windows_directory()
        .and_then(|directory| source.volume_path_for(&directory))
        .and_then(|root| source.volume_serial(&root))
        .ok();

    let host_id = derive_host_id(machine_guid.as_deref(), volume_serial)?;
    Ok(HostIdentity {
        host_id,
        signals: HostIdentitySignals {
            machine_guid_available: machine_guid.is_some(),
            system_volume_serial_available: volume_serial.is_some(),
        },
    })
}

fn derive_host_id(
    machine_guid: Option<&str>,
    volume_serial: Option<u32>,
) -> Result<String, String> {
    let machine_guid = machine_guid
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (machine_guid, volume_serial) {
        (Some(machine_guid), Some(volume_serial)) => Ok(format!(
            "host-v1-{}",
            &stable_hash(&format!(
                "edy-sentinel-host-v1|{}|{volume_serial:08x}",
                machine_guid.to_lowercase()
            ))[..32]
        )),
        (Some(machine_guid), None) => Ok(format!(
            "host-guid-v1-{}",
            &stable_hash(&format!(
                "edy-sentinel-host-guid-only-v1|{}",
                machine_guid.to_lowercase()
            ))[..32]
        )),
        (None, Some(volume_serial)) => Ok(format!(
            "host-volume-v1-{}",
            &stable_hash(&format!(
                "edy-sentinel-host-volume-only-v1|{volume_serial:08x}"
            ))[..32]
        )),
        (None, None) => Err("Windows host identity is unavailable".into()),
    }
}

fn stable_hash(value: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(value.as_bytes());
    format!("{:x}", digest.finalize())
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn decode_null_terminated(buffer: &[u16], error: &str) -> Result<String, String> {
    let length = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    if length == 0 {
        return Err(error.into());
    }
    String::from_utf16(&buffer[..length]).map_err(|_| error.into())
}

fn ensure_trailing_separator(mut value: String) -> String {
    if !value.ends_with('\\') {
        value.push('\\');
    }
    value
}

#[cfg(test)]
mod tests {
    use super::{collect_host_identity, derive_host_id, HostIdentitySource};
    use std::cell::RefCell;

    struct FakeHostIdentitySource {
        machine_guid: Option<String>,
        windows_directory: Result<String, String>,
        volume_root: Result<String, String>,
        volume_serial: Result<u32, String>,
        observed_paths: RefCell<Vec<String>>,
        observed_roots: RefCell<Vec<String>>,
    }

    impl FakeHostIdentitySource {
        fn available(windows_directory: &str, volume_root: &str) -> Self {
            Self {
                machine_guid: Some("machine-guid-value".into()),
                windows_directory: Ok(windows_directory.into()),
                volume_root: Ok(volume_root.into()),
                volume_serial: Ok(0x1234_abcd),
                observed_paths: RefCell::new(Vec::new()),
                observed_roots: RefCell::new(Vec::new()),
            }
        }
    }

    impl HostIdentitySource for FakeHostIdentitySource {
        fn machine_guid(&self) -> Option<String> {
            self.machine_guid.clone()
        }

        fn system_windows_directory(&self) -> Result<String, String> {
            self.windows_directory.clone()
        }

        fn volume_path_for(&self, path: &str) -> Result<String, String> {
            self.observed_paths.borrow_mut().push(path.into());
            self.volume_root.clone()
        }

        fn volume_serial(&self, volume_root: &str) -> Result<u32, String> {
            self.observed_roots.borrow_mut().push(volume_root.into());
            self.volume_serial.clone()
        }
    }

    #[test]
    fn preserves_v1_derivation_when_both_signals_exist() {
        assert_eq!(
            derive_host_id(Some("machine-guid-value"), Some(0x1234_abcd)).unwrap(),
            "host-v1-5f12955098476f61df40954dc96681d2"
        );
    }

    #[test]
    fn resolves_windows_installed_on_c() {
        let source = FakeHostIdentitySource::available("C:\\Windows", "C:\\");
        let identity = collect_host_identity(&source).unwrap();

        assert_eq!(source.observed_paths.borrow().as_slice(), ["C:\\Windows"]);
        assert_eq!(source.observed_roots.borrow().as_slice(), ["C:\\"]);
        assert!(identity.signals.machine_guid_available);
        assert!(identity.signals.system_volume_serial_available);
    }

    #[test]
    fn resolves_windows_installed_on_another_volume() {
        let source = FakeHostIdentitySource::available("D:\\Windows", "D:\\");
        collect_host_identity(&source).unwrap();

        assert_eq!(source.observed_paths.borrow().as_slice(), ["D:\\Windows"]);
        assert_eq!(source.observed_roots.borrow().as_slice(), ["D:\\"]);
    }

    #[test]
    fn falls_back_to_machine_guid_when_volume_serial_fails() {
        let mut source = FakeHostIdentitySource::available("D:\\Windows", "D:\\");
        source.volume_serial = Err("native failure with sensitive detail".into());

        let identity = collect_host_identity(&source).unwrap();
        assert!(identity.host_id.starts_with("host-guid-v1-"));
        assert!(identity.signals.machine_guid_available);
        assert!(!identity.signals.system_volume_serial_available);
        assert!(!identity.host_id.contains("machine-guid-value"));
        assert!(!identity.host_id.contains("sensitive"));
    }

    #[test]
    fn falls_back_to_volume_serial_when_machine_guid_is_absent() {
        let mut source = FakeHostIdentitySource::available("C:\\Windows", "C:\\");
        source.machine_guid = None;

        let identity = collect_host_identity(&source).unwrap();
        assert!(identity.host_id.starts_with("host-volume-v1-"));
        assert!(!identity.signals.machine_guid_available);
        assert!(identity.signals.system_volume_serial_available);
        assert!(!identity.host_id.contains("1234abcd"));
    }

    #[test]
    fn rejects_identity_when_both_signals_are_absent() {
        let source = FakeHostIdentitySource {
            machine_guid: None,
            windows_directory: Err("directory unavailable".into()),
            volume_root: Err("volume unavailable".into()),
            volume_serial: Err("serial unavailable".into()),
            observed_paths: RefCell::new(Vec::new()),
            observed_roots: RefCell::new(Vec::new()),
        };

        assert_eq!(
            collect_host_identity(&source).unwrap_err(),
            "Windows host identity is unavailable"
        );
    }

    #[test]
    fn partial_identities_are_stable_and_domain_separated() {
        let guid_first = derive_host_id(Some(" Stable-Guid "), None).unwrap();
        let guid_second = derive_host_id(Some("stable-guid"), None).unwrap();
        let serial_first = derive_host_id(None, Some(42)).unwrap();
        let serial_second = derive_host_id(None, Some(42)).unwrap();

        assert_eq!(guid_first, guid_second);
        assert_eq!(serial_first, serial_second);
        assert_ne!(guid_first, serial_first);
        assert!(!guid_first.contains("stable-guid"));
        assert!(!serial_first.contains("0000002a"));
    }
}
