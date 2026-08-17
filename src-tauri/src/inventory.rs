use crate::{
    models::{InstalledSoftwareRecord, SoftwareIdentity, SoftwareInventorySnapshot},
    persistence::Database,
    vulnerability_matching,
};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, time::Instant};
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY},
    RegKey,
};

const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
const UNRESOLVED: &str = "Unresolved";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryScope {
    Machine,
    User,
}

impl RegistryScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::User => "user",
        }
    }

    fn root_name(self) -> &'static str {
        match self {
            Self::Machine => "HKLM",
            Self::User => "HKCU",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryView {
    Native64,
    Wow32,
}

impl RegistryView {
    fn source(self, scope: RegistryScope) -> String {
        format!(
            "windows_registry_{}_{}",
            scope.as_str(),
            match self {
                Self::Native64 => "64",
                Self::Wow32 => "32",
            }
        )
    }

    fn flag(self) -> u32 {
        match self {
            Self::Native64 => KEY_WOW64_64KEY,
            Self::Wow32 => KEY_WOW64_32KEY,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawSoftwareEntry {
    display_name: String,
    display_version: Option<String>,
    publisher: Option<String>,
    install_location: Option<String>,
    install_date: Option<String>,
    architecture: String,
    install_scope: String,
    source: String,
    registry_identity: String,
    product_code: Option<String>,
}

pub(crate) fn refresh_inventory(database: &Database) -> Result<SoftwareInventorySnapshot, String> {
    let collected = collect_installed_software()?;
    persist_inventory(database, &collected)?;
    let items = load_inventory(database)?;
    Ok(SoftwareInventorySnapshot { items, ..collected })
}

pub(crate) fn current_inventory(
    database: &Database,
) -> Result<Vec<InstalledSoftwareRecord>, String> {
    load_inventory(database)
}

fn collect_installed_software() -> Result<SoftwareInventorySnapshot, String> {
    let started = Instant::now();
    let now = Utc::now().to_rfc3339();
    let mut raw = Vec::new();
    let mut source_count = 0;
    for (scope, view) in registry_sources() {
        let root = RegKey::predef(match scope {
            RegistryScope::Machine => HKEY_LOCAL_MACHINE,
            RegistryScope::User => HKEY_CURRENT_USER,
        });
        let Ok(uninstall) = root.open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | view.flag())
        else {
            continue;
        };
        source_count += 1;
        for subkey_name in uninstall.enum_keys().filter_map(Result::ok) {
            let Ok(key) = uninstall.open_subkey_with_flags(&subkey_name, KEY_READ) else {
                continue;
            };
            if key.get_value::<u32, _>("SystemComponent").ok() == Some(1) {
                continue;
            }
            let Some(display_name) = registry_string(&key, "DisplayName") else {
                continue;
            };
            let product_code = normalize_product_code(&subkey_name);
            raw.push(RawSoftwareEntry {
                display_name,
                display_version: registry_string(&key, "DisplayVersion"),
                publisher: registry_string(&key, "Publisher"),
                install_location: registry_string(&key, "InstallLocation"),
                install_date: registry_string(&key, "InstallDate"),
                architecture: architecture_for_view(view),
                install_scope: scope.as_str().into(),
                source: view.source(scope),
                registry_identity: format!(
                    r"{}\{}\{}\{}",
                    scope.root_name(),
                    match view {
                        RegistryView::Native64 => "64",
                        RegistryView::Wow32 => "32",
                    },
                    UNINSTALL_PATH,
                    subkey_name
                ),
                product_code,
            });
        }
    }
    if source_count == 0 {
        return Err("Software inventory sources are unavailable".into());
    }
    let raw_entry_count = raw.len();
    let items = deduplicate(raw, &now);
    Ok(SoftwareInventorySnapshot {
        items,
        collected_at: now,
        duration_ms: started.elapsed().as_millis() as u64,
        raw_entry_count,
        source_count,
    })
}

fn registry_sources() -> [(RegistryScope, RegistryView); 4] {
    [
        (RegistryScope::Machine, RegistryView::Native64),
        (RegistryScope::Machine, RegistryView::Wow32),
        (RegistryScope::User, RegistryView::Native64),
        (RegistryScope::User, RegistryView::Wow32),
    ]
}

fn registry_string(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn architecture_for_view(view: RegistryView) -> String {
    match (std::env::consts::ARCH, view) {
        ("x86_64", RegistryView::Native64) => "x64".into(),
        ("x86_64" | "x86", RegistryView::Wow32) => "x86".into(),
        ("aarch64", RegistryView::Native64) => "arm64".into(),
        _ => "unresolved".into(),
    }
}

fn normalize_product_code(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() != 38 || !value.starts_with('{') || !value.ends_with('}') {
        return None;
    }
    let body = &value[1..37];
    if body
        .chars()
        .enumerate()
        .all(|(index, character)| match index {
            8 | 13 | 18 | 23 => character == '-',
            _ => character.is_ascii_hexdigit(),
        })
    {
        Some(value.to_ascii_uppercase())
    } else {
        None
    }
}

fn deduplicate(entries: Vec<RawSoftwareEntry>, now: &str) -> Vec<InstalledSoftwareRecord> {
    let mut grouped: BTreeMap<String, Vec<RawSoftwareEntry>> = BTreeMap::new();
    for entry in entries {
        let key = entry.product_code.as_ref().map_or_else(
            || format!("registry:{}", entry.registry_identity.to_ascii_lowercase()),
            |code| format!("product-code:{}:{}", entry.install_scope, code),
        );
        grouped.entry(key).or_default().push(entry);
    }
    grouped
        .into_iter()
        .map(|(identity_key, mut entries)| {
            entries.sort_by_key(completeness_score);
            let primary = entries
                .last()
                .expect("software group cannot be empty")
                .clone();
            let mut sources: Vec<_> = entries.iter().map(|entry| entry.source.clone()).collect();
            sources.sort();
            sources.dedup();
            let mut registry_identities: Vec<_> = entries
                .iter()
                .map(|entry| entry.registry_identity.clone())
                .collect();
            registry_identities.sort();
            registry_identities.dedup();
            let vendor = primary
                .publisher
                .clone()
                .unwrap_or_else(|| UNRESOLVED.into());
            let product = primary
                .product_code
                .as_ref()
                .map(|_| primary.display_name.clone())
                .unwrap_or_else(|| UNRESOLVED.into());
            let version = primary
                .display_version
                .clone()
                .unwrap_or_else(|| UNRESOLVED.into());
            let status = if primary.product_code.is_some()
                && version != UNRESOLVED
                && vendor != UNRESOLVED
                && primary.architecture != "unresolved"
            {
                "resolved"
            } else {
                "unresolved"
            };
            InstalledSoftwareRecord {
                software_id: stable_id(&identity_key),
                display_name: primary.display_name,
                display_version: primary.display_version,
                publisher: primary.publisher,
                install_location: primary.install_location,
                install_date: primary.install_date,
                architecture: primary.architecture.clone(),
                install_scope: primary.install_scope.clone(),
                sources,
                registry_identities,
                product_code: primary.product_code,
                normalized_identity: SoftwareIdentity {
                    vendor,
                    product,
                    version,
                    architecture: primary.architecture,
                    install_scope: primary.install_scope,
                    status: status.into(),
                },
                first_seen_at: now.into(),
                last_seen_at: now.into(),
                observation_count: 1,
                active: true,
            }
        })
        .collect()
}

fn completeness_score(entry: &RawSoftwareEntry) -> usize {
    [
        entry.display_version.is_some(),
        entry.publisher.is_some(),
        entry.install_location.is_some(),
        entry.install_date.is_some(),
        entry.product_code.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
}

fn stable_id(material: &str) -> String {
    let digest = Sha256::digest(format!("software-inventory-v1|{material}").as_bytes());
    format!("software-v1-{}", hex(&digest[..16]))
}

fn event_id(event_type: &str, software_id: &str, occurred_at: &str) -> String {
    let digest = Sha256::digest(
        format!("software-event-v1|{event_type}|{software_id}|{occurred_at}").as_bytes(),
    );
    format!("software-event-v1-{}", hex(&digest[..16]))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn persist_inventory(
    database: &Database,
    snapshot: &SoftwareInventorySnapshot,
) -> Result<(), String> {
    database.repository_transaction(|transaction| {
        let snapshot_id = stable_id(&format!("snapshot|{}", snapshot.collected_at));
        let previous_snapshot_count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM software_inventory_snapshots",
            [],
            |row| row.get(0),
        )?;
        let mut previous = BTreeMap::<String, (Option<String>, String)>::new();
        {
            let mut statement = transaction.prepare(
                "SELECT software_id, display_version, display_name FROM installed_software WHERE active = 1",
            )?;
            let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            for row in rows {
                let (id, version, name) = row?;
                previous.insert(id, (version, name));
            }
        }
        transaction.execute(
            "INSERT INTO software_inventory_snapshots(
                snapshot_id, collected_at, duration_ms, raw_entry_count, software_count, source_count
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                snapshot_id,
                snapshot.collected_at,
                snapshot.duration_ms,
                snapshot.raw_entry_count,
                snapshot.items.len(),
                snapshot.source_count,
            ],
        )?;
        transaction.execute("UPDATE installed_software SET active = 0", [])?;
        for item in &snapshot.items {
            let sources = serde_json::to_string(&item.sources)
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
            let identities = serde_json::to_string(&item.registry_identities)
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
            transaction.execute(
                "INSERT INTO installed_software(
                    software_id, display_name, display_version, publisher, install_location,
                    install_date, architecture, install_scope, sources_json,
                    registry_identities_json, product_code, normalized_vendor,
                    normalized_product, normalized_version, identity_status, first_seen_at,
                    last_seen_at, observation_count, active, last_snapshot_id
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                    ?15, ?16, ?17, 1, 1, ?18
                 ) ON CONFLICT(software_id) DO UPDATE SET
                    display_name=excluded.display_name, display_version=excluded.display_version,
                    publisher=excluded.publisher, install_location=excluded.install_location,
                    install_date=excluded.install_date, architecture=excluded.architecture,
                    install_scope=excluded.install_scope, sources_json=excluded.sources_json,
                    registry_identities_json=excluded.registry_identities_json,
                    product_code=excluded.product_code, normalized_vendor=excluded.normalized_vendor,
                    normalized_product=excluded.normalized_product,
                    normalized_version=excluded.normalized_version,
                    identity_status=excluded.identity_status, last_seen_at=excluded.last_seen_at,
                    observation_count=installed_software.observation_count + 1, active=1,
                    last_snapshot_id=excluded.last_snapshot_id",
                params![
                    item.software_id,
                    item.display_name,
                    item.display_version,
                    item.publisher,
                    item.install_location,
                    item.install_date,
                    item.architecture,
                    item.install_scope,
                    sources,
                    identities,
                    item.product_code,
                    item.normalized_identity.vendor,
                    item.normalized_identity.product,
                    item.normalized_identity.version,
                    item.normalized_identity.status,
                    snapshot.collected_at,
                    snapshot.collected_at,
                    snapshot_id,
                ],
            )?;
            let evidence = serde_json::json!({
                "displayName": item.display_name,
                "displayVersion": item.display_version,
                "publisher": item.publisher,
                "architecture": item.architecture,
                "installScope": item.install_scope,
                "sources": item.sources,
                "registryIdentities": item.registry_identities,
                "productCode": item.product_code,
            });
            transaction.execute(
                "INSERT INTO software_inventory_observations(
                    snapshot_id, software_id, observed_at, display_version, publisher,
                    architecture, install_scope, evidence_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    snapshot_id,
                    item.software_id,
                    snapshot.collected_at,
                    item.display_version,
                    item.publisher,
                    item.architecture,
                    item.install_scope,
                    evidence.to_string(),
                ],
            )?;
            if previous_snapshot_count > 0 {
                match previous.remove(&item.software_id) {
                    None => {
                        insert_factual_event(
                            transaction,
                            "software_installed",
                            item,
                            &snapshot.collected_at,
                            serde_json::json!({ "displayName": item.display_name, "version": item.display_version }),
                        )?;
                        vulnerability_matching::enqueue(
                            transaction,
                            &item.software_id,
                            "software_installed",
                            &snapshot.collected_at,
                        )?;
                    }
                    Some((before, _)) if before != item.display_version => {
                        insert_factual_event(
                            transaction,
                            "software_version_changed",
                            item,
                            &snapshot.collected_at,
                            serde_json::json!({ "displayName": item.display_name, "beforeVersion": before, "afterVersion": item.display_version }),
                        )?;
                        vulnerability_matching::enqueue(
                            transaction,
                            &item.software_id,
                            "software_version_changed",
                            &snapshot.collected_at,
                        )?;
                    }
                    _ => {}
                }
            } else {
                previous.remove(&item.software_id);
                vulnerability_matching::enqueue(
                    transaction,
                    &item.software_id,
                    "initial",
                    &snapshot.collected_at,
                )?;
            }
        }
        if previous_snapshot_count > 0 {
            for (software_id, (version, display_name)) in previous {
                let item = load_one(transaction, &software_id)?
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                insert_factual_event(
                    transaction,
                    "software_removed",
                    &item,
                    &snapshot.collected_at,
                    serde_json::json!({ "displayName": display_name, "lastKnownVersion": version }),
                )?;
            }
        }
        Ok(())
    })
    .map_err(|_| "Unable to persist software inventory".into())
}

fn insert_factual_event(
    transaction: &rusqlite::Transaction<'_>,
    event_type: &str,
    item: &InstalledSoftwareRecord,
    occurred_at: &str,
    evidence: serde_json::Value,
) -> Result<(), rusqlite::Error> {
    let id = event_id(event_type, &item.software_id, occurred_at);
    let evidence_json = evidence.to_string();
    let title = match event_type {
        "software_installed" => "Software installed",
        "software_removed" => "Software removed",
        _ => "Installed software version changed",
    };
    transaction.execute(
        "INSERT INTO software_inventory_events(event_id, event_type, software_id, occurred_at, evidence_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, event_type, item.software_id, occurred_at, evidence_json],
    )?;
    transaction.execute(
        "INSERT INTO security_events(
            id, event_type, occurred_at, source, payload_json, entity_type, entity_key,
            title, first_seen_at, last_seen_at, baseline_context_json, status,
            observation_count, condition_active, schema_version
         ) VALUES (?1, ?2, ?3, 'software_inventory', ?4, 'software', ?5, ?6, ?3, ?3, '{}', 'new', 1, 0, 1)",
        params![id, event_type, occurred_at, evidence_json, item.software_id, title],
    )?;
    transaction.execute(
        "INSERT INTO security_event_history(
            event_id, transition, observed_at, recorded_at, source, event_type,
            entity_type, entity_key, evidence_json, baseline_context_json,
            event_schema_version, history_schema_version, observation_count
         ) VALUES (?1, 'first_observed', ?2, ?2, 'software_inventory', ?3,
            'software', ?4, ?5, '{}', 1, 1, 1)",
        params![id, occurred_at, event_type, item.software_id, evidence_json],
    )?;
    Ok(())
}

fn load_inventory(database: &Database) -> Result<Vec<InstalledSoftwareRecord>, String> {
    database
        .repository_read(|connection| {
            let mut statement = connection.prepare(
                "SELECT software_id FROM installed_software WHERE active = 1
                 ORDER BY display_name COLLATE NOCASE, software_id",
            )?;
            let ids = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            ids.into_iter()
                .map(|id| load_one(connection, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows))
                .collect()
        })
        .map_err(|_| "Unable to load software inventory".into())
}

fn load_one(
    connection: &rusqlite::Connection,
    software_id: &str,
) -> Result<Option<InstalledSoftwareRecord>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT software_id, display_name, display_version, publisher, install_location,
                    install_date, architecture, install_scope, sources_json,
                    registry_identities_json, product_code, normalized_vendor,
                    normalized_product, normalized_version, identity_status, first_seen_at,
                    last_seen_at, observation_count, active
             FROM installed_software WHERE software_id = ?1",
            [software_id],
            |row| {
                let sources_json: String = row.get(8)?;
                let identities_json: String = row.get(9)?;
                Ok(InstalledSoftwareRecord {
                    software_id: row.get(0)?,
                    display_name: row.get(1)?,
                    display_version: row.get(2)?,
                    publisher: row.get(3)?,
                    install_location: row.get(4)?,
                    install_date: row.get(5)?,
                    architecture: row.get(6)?,
                    install_scope: row.get(7)?,
                    sources: serde_json::from_str(&sources_json).unwrap_or_default(),
                    registry_identities: serde_json::from_str(&identities_json).unwrap_or_default(),
                    product_code: row.get(10)?,
                    normalized_identity: SoftwareIdentity {
                        vendor: row.get(11)?,
                        product: row.get(12)?,
                        version: row.get(13)?,
                        architecture: row.get(6)?,
                        install_scope: row.get(7)?,
                        status: row.get(14)?,
                    },
                    first_seen_at: row.get(15)?,
                    last_seen_at: row.get(16)?,
                    observation_count: row.get(17)?,
                    active: row.get::<_, i64>(18)? != 0,
                })
            },
        )
        .optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(identity: &str, name: &str) -> RawSoftwareEntry {
        RawSoftwareEntry {
            display_name: name.into(),
            display_version: Some("1.0".into()),
            publisher: Some("Example Vendor".into()),
            install_location: None,
            install_date: None,
            architecture: "x64".into(),
            install_scope: "machine".into(),
            source: "windows_registry_machine_64".into(),
            registry_identity: identity.into(),
            product_code: None,
        }
    }

    #[test]
    fn registry_sources_cover_machine_user_and_both_views() {
        assert_eq!(registry_sources().len(), 4);
        assert!(registry_sources().contains(&(RegistryScope::Machine, RegistryView::Native64)));
        assert!(registry_sources().contains(&(RegistryScope::Machine, RegistryView::Wow32)));
        assert!(registry_sources().contains(&(RegistryScope::User, RegistryView::Native64)));
        assert!(registry_sources().contains(&(RegistryScope::User, RegistryView::Wow32)));
    }

    #[test]
    fn deduplication_requires_product_code_and_same_scope() {
        let mut first = entry("HKLM\\64\\one", "Example");
        first.product_code = Some("{12345678-1234-1234-1234-123456789ABC}".into());
        let mut duplicate = first.clone();
        duplicate.registry_identity = "HKLM\\32\\two".into();
        duplicate.source = "windows_registry_machine_32".into();
        let mut user_copy = duplicate.clone();
        user_copy.install_scope = "user".into();
        assert_eq!(
            deduplicate(vec![first, duplicate, user_copy], "now").len(),
            2
        );
        assert_eq!(
            deduplicate(vec![entry("a", "Same"), entry("b", "Same")], "now").len(),
            2
        );
    }

    #[test]
    fn normalization_is_conservative_when_product_identity_is_absent() {
        let item = deduplicate(vec![entry("a", "Friendly Name")], "now").remove(0);
        assert_eq!(item.normalized_identity.product, UNRESOLVED);
        assert_eq!(item.normalized_identity.status, "unresolved");
        assert_eq!(item.display_name, "Friendly Name");
    }

    #[test]
    fn product_codes_are_strict_and_versions_are_preserved() {
        assert_eq!(
            normalize_product_code("{12345678-1234-ABCD-9876-1234567890ab}"),
            Some("{12345678-1234-ABCD-9876-1234567890AB}".into())
        );
        assert_eq!(normalize_product_code("Example Product"), None);
        let item = deduplicate(vec![entry("a", "Example")], "now").remove(0);
        assert_eq!(item.display_version.as_deref(), Some("1.0"));
    }

    #[test]
    fn persistence_tracks_install_remove_and_version_changes_without_severity() {
        let database = Database::in_memory().expect("database");
        let mut first = SoftwareInventorySnapshot {
            items: deduplicate(vec![entry("a", "Example")], "2026-01-01T00:00:00Z"),
            collected_at: "2026-01-01T00:00:00Z".into(),
            duration_ms: 1,
            raw_entry_count: 1,
            source_count: 4,
        };
        persist_inventory(&database, &first).expect("seed inventory");
        first.items[0].display_version = Some("2.0".into());
        first.items[0].normalized_identity.version = "2.0".into();
        first.collected_at = "2026-01-02T00:00:00Z".into();
        persist_inventory(&database, &first).expect("version inventory");
        let empty = SoftwareInventorySnapshot {
            items: vec![],
            collected_at: "2026-01-03T00:00:00Z".into(),
            duration_ms: 1,
            raw_entry_count: 0,
            source_count: 4,
        };
        persist_inventory(&database, &empty).expect("removed inventory");
        database
            .repository_read(|connection| {
                let events: Vec<String> = connection
                    .prepare("SELECT event_type FROM software_inventory_events ORDER BY occurred_at")?
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<_, _>>()?;
                assert_eq!(events, ["software_version_changed", "software_removed"]);
                let severity_column: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('software_inventory_events') WHERE name='severity'",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(severity_column, 0);
                Ok(())
            })
            .expect("events");
    }

    #[test]
    fn native_windows_inventory_smoke() {
        let snapshot = collect_installed_software().expect("native inventory");
        eprintln!(
            "inventory_smoke software={} raw={} sources={} duration_ms={}",
            snapshot.items.len(),
            snapshot.raw_entry_count,
            snapshot.source_count,
            snapshot.duration_ms
        );
        assert!(snapshot.source_count >= 2);
        assert!(!snapshot.items.is_empty());
    }
}
