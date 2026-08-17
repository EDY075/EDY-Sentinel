use crate::{
    baseline::BaselineEngine,
    collectors,
    detection::DetectionEngine,
    detection_query::{
        query_detection_evidence, query_detections, DetectionEvidenceInput, DetectionEvidencePage,
        DetectionPage, DetectionQueryInput,
    },
    event_query::{
        query_security_event_history, query_security_events, SecurityEventHistoryInput,
        SecurityEventHistoryPage, SecurityEventPage, SecurityEventQueryInput,
    },
    inventory,
    models::{
        BaselineActionInput, BaselineSummary, Capability, CollectionIssue, CollectorHealth,
        CollectorStatus, DatabaseStatus, DetectionStatusInput, InstalledSoftwareRecord,
        LanguageInput, LiveTelemetrySnapshot, RuleEnabledInput, ScoreCoverage, SecurityEventRecord,
        SecurityEventStatusInput, SecurityScore, SoftwareInventorySnapshot,
        SoftwareVulnerabilityDetail, SoftwareVulnerabilityInput, SoftwareVulnerabilitySummary,
        SystemOverview, ThemeInput, VulnerabilityEvaluationInput, VulnerabilityEvaluationSummary,
        VulnerabilityProviderStatus, VulnerabilitySyncInput,
    },
    persistence::Database,
    rules::{self, RuleDefinition},
    score::ScoreEngine,
    telemetry::TelemetryEngine,
    vulnerability::{self, VulnerabilitySyncManager},
    vulnerability_matching,
};
use std::sync::Arc;
use tauri::State;

const THEMES: [&str; 4] = ["sentinel-blue", "cyber-green", "terminal", "spectrum"];
const LANGUAGES: [&str; 2] = ["pt-BR", "en"];

#[tauri::command]
pub fn get_system_locale() -> Result<String, String> {
    const LOCALE_NAME_CAPACITY: usize = 85;
    let mut locale_name = [0_u16; LOCALE_NAME_CAPACITY];
    let length = unsafe {
        windows_sys::Win32::Globalization::GetUserDefaultLocaleName(
            locale_name.as_mut_ptr(),
            locale_name.len() as i32,
        )
    };
    if length == 0 {
        return Err("Unable to read the Windows user locale".into());
    }
    String::from_utf16(&locale_name[..length.saturating_sub(1) as usize])
        .map_err(|_| "Windows returned an invalid user locale".into())
}

#[tauri::command]
pub async fn get_system_overview(
    database: State<'_, Database>,
    baseline: State<'_, BaselineEngine>,
    detection: State<'_, DetectionEngine>,
    score: State<'_, ScoreEngine>,
) -> Result<SystemOverview, String> {
    let mut overview = tauri::async_runtime::spawn_blocking(collectors::collect_overview)
        .await
        .map_err(|_| "System collection task failed".to_string())??;
    if database.save_snapshot(&overview).is_err() {
        overview.issues.push(CollectionIssue {
            component: "persistence".into(),
            message: "Telemetry was collected, but the local snapshot could not be saved".into(),
        });
    }
    if let Err(message) = baseline.observe_network(&database, &overview) {
        overview.issues.push(CollectionIssue {
            component: "baseline".into(),
            message,
        });
    }
    let coverage = vec![collector_coverage(&overview.collector)];
    refresh_security_analysis(
        &database,
        &detection,
        &score,
        coverage,
        &mut overview.issues,
    );
    Ok(overview)
}

#[tauri::command]
pub fn get_theme(database: State<'_, Database>) -> Result<String, String> {
    database.get_theme()
}

#[tauri::command]
pub fn set_theme(input: ThemeInput, database: State<'_, Database>) -> Result<(), String> {
    if !THEMES.contains(&input.theme.as_str()) {
        return Err("Unsupported theme".into());
    }
    database.set_theme(&input.theme)
}

#[tauri::command]
pub fn get_software_inventory(
    database: State<'_, Database>,
) -> Result<Vec<InstalledSoftwareRecord>, String> {
    inventory::current_inventory(&database)
}

#[tauri::command]
pub async fn refresh_software_inventory(
    database: State<'_, Database>,
) -> Result<SoftwareInventorySnapshot, String> {
    let database = database.inner().clone();
    tauri::async_runtime::spawn_blocking(move || inventory::refresh_inventory(&database))
        .await
        .map_err(|_| "Software inventory task failed".to_string())?
}

#[tauri::command]
pub fn get_vulnerability_provider_status(
    database: State<'_, Database>,
) -> Result<Vec<VulnerabilityProviderStatus>, String> {
    vulnerability::provider_statuses(&database)
}

#[tauri::command]
pub async fn sync_vulnerability_provider(
    input: VulnerabilitySyncInput,
    database: State<'_, Database>,
    manager: State<'_, Arc<VulnerabilitySyncManager>>,
) -> Result<VulnerabilityProviderStatus, String> {
    manager.begin()?;
    let database = database.inner().clone();
    let manager = manager.inner().clone();
    let provider = input.provider;
    tauri::async_runtime::spawn_blocking(move || {
        let result = vulnerability::sync_provider(&database, &manager, &provider);
        manager.finish();
        result
    })
    .await
    .map_err(|_| "Vulnerability repository sync task failed".to_string())?
}

#[tauri::command]
pub fn cancel_vulnerability_sync(manager: State<'_, Arc<VulnerabilitySyncManager>>) {
    manager.cancel();
}

#[tauri::command]
pub fn get_software_vulnerability_summaries(
    database: State<'_, Database>,
) -> Result<Vec<SoftwareVulnerabilitySummary>, String> {
    vulnerability_matching::summaries(&database)
}

#[tauri::command]
pub async fn evaluate_software_vulnerabilities(
    input: VulnerabilityEvaluationInput,
    database: State<'_, Database>,
) -> Result<VulnerabilityEvaluationSummary, String> {
    let database = database.inner().clone();
    tauri::async_runtime::spawn_blocking(move || vulnerability_matching::evaluate(&database, input))
        .await
        .map_err(|_| "Vulnerability evaluation task failed".to_string())?
}

#[tauri::command]
pub fn get_software_vulnerability_detail(
    input: SoftwareVulnerabilityInput,
    database: State<'_, Database>,
) -> Result<SoftwareVulnerabilityDetail, String> {
    vulnerability_matching::detail(&database, &input.software_id)
}

#[tauri::command]
pub fn get_language(database: State<'_, Database>) -> Result<Option<String>, String> {
    database.get_language()
}

#[tauri::command]
pub fn set_language(input: LanguageInput, database: State<'_, Database>) -> Result<(), String> {
    if !LANGUAGES.contains(&input.language.as_str()) {
        return Err("Unsupported language".into());
    }
    database.set_language(&input.language)
}

#[tauri::command]
pub fn get_database_status(database: State<'_, Database>) -> Result<DatabaseStatus, String> {
    let (schema_version, writable) = database.status()?;
    Ok(DatabaseStatus {
        schema_version,
        path_kind: "application-data".into(),
        writable,
    })
}

#[tauri::command]
pub async fn get_live_telemetry(
    engine: State<'_, TelemetryEngine>,
    database: State<'_, Database>,
    baseline: State<'_, BaselineEngine>,
    detection: State<'_, DetectionEngine>,
    score: State<'_, ScoreEngine>,
) -> Result<LiveTelemetrySnapshot, String> {
    let engine = engine.inner().clone();
    let database = database.inner().clone();
    let baseline = baseline.inner().clone();
    let detection = detection.inner().clone();
    let score = score.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut snapshot = engine.collect()?;
        if database.persist_live_telemetry(&mut snapshot).is_err() {
            snapshot.issues.push(CollectionIssue {
                component: "persistence".into(),
                message: "Live telemetry was collected, but local observation tracking could not be saved".into(),
            });
        }
        if let Err(message) = baseline.observe_live(&database, &snapshot) {
            snapshot.issues.push(CollectionIssue {
                component: "baseline".into(),
                message,
            });
        }
        let coverage = snapshot.collectors.iter().map(collector_coverage).collect();
        refresh_security_analysis(
            &database,
            &detection,
            &score,
            coverage,
            &mut snapshot.issues,
        );
        Ok(snapshot)
    })
    .await
    .map_err(|_| "Live telemetry task failed".to_string())?
}

#[tauri::command]
pub fn get_baseline_summary(
    baseline: State<'_, BaselineEngine>,
    database: State<'_, Database>,
) -> Result<BaselineSummary, String> {
    baseline.summary(&database)
}

#[tauri::command]
pub fn get_security_events(
    baseline: State<'_, BaselineEngine>,
    database: State<'_, Database>,
) -> Result<Vec<SecurityEventRecord>, String> {
    baseline.security_events(&database)
}

#[tauri::command]
pub fn get_security_events_page(
    input: SecurityEventQueryInput,
    database: State<'_, Database>,
) -> Result<SecurityEventPage, String> {
    query_security_events(&database, input)
}

#[tauri::command]
pub fn get_security_event_history(
    input: SecurityEventHistoryInput,
    database: State<'_, Database>,
) -> Result<SecurityEventHistoryPage, String> {
    query_security_event_history(&database, input)
}

#[tauri::command]
pub fn start_new_baseline(
    input: BaselineActionInput,
    baseline: State<'_, BaselineEngine>,
    database: State<'_, Database>,
) -> Result<BaselineSummary, String> {
    baseline.start_new_baseline(&database, input)
}

#[tauri::command]
pub fn reset_baseline(
    input: BaselineActionInput,
    baseline: State<'_, BaselineEngine>,
    database: State<'_, Database>,
) -> Result<BaselineSummary, String> {
    baseline.reset_baseline(&database, input)
}

#[tauri::command]
pub fn complete_baseline_learning(
    input: BaselineActionInput,
    baseline: State<'_, BaselineEngine>,
    database: State<'_, Database>,
) -> Result<BaselineSummary, String> {
    baseline.complete_learning(&database, input)
}

#[tauri::command]
pub fn set_security_event_status(
    input: SecurityEventStatusInput,
    baseline: State<'_, BaselineEngine>,
    database: State<'_, Database>,
) -> Result<(), String> {
    baseline.set_event_status(&database, input)
}

#[tauri::command]
pub fn get_detections_page(
    input: DetectionQueryInput,
    database: State<'_, Database>,
) -> Result<DetectionPage, String> {
    query_detections(&database, input)
}

#[tauri::command]
pub fn get_detection_evidence(
    input: DetectionEvidenceInput,
    database: State<'_, Database>,
) -> Result<DetectionEvidencePage, String> {
    query_detection_evidence(&database, input)
}

#[tauri::command]
pub fn set_detection_status(
    input: DetectionStatusInput,
    detection: State<'_, DetectionEngine>,
    score: State<'_, ScoreEngine>,
    database: State<'_, Database>,
) -> Result<(), String> {
    detection.set_detection_status(&database, input)?;
    score.calculate_and_persist(&database, Vec::new())?;
    Ok(())
}

#[tauri::command]
pub fn get_detection_rules(
    detection: State<'_, DetectionEngine>,
    database: State<'_, Database>,
) -> Result<Vec<RuleDefinition>, String> {
    detection.rule_definitions(&database)
}

#[tauri::command]
pub fn set_detection_rule_enabled(
    input: RuleEnabledInput,
    detection: State<'_, DetectionEngine>,
    score: State<'_, ScoreEngine>,
    database: State<'_, Database>,
) -> Result<(), String> {
    detection.set_rule_enabled(&database, &input.rule_id, input.enabled)?;
    score.calculate_and_persist(&database, Vec::new())?;
    Ok(())
}

#[tauri::command]
pub fn get_security_score(
    score: State<'_, ScoreEngine>,
    database: State<'_, Database>,
) -> Result<SecurityScore, String> {
    score.current(&database)
}

fn collector_coverage(collector: &CollectorHealth) -> ScoreCoverage {
    ScoreCoverage {
        component: collector.id.clone(),
        status: match collector.status {
            CollectorStatus::Healthy => "healthy",
            CollectorStatus::Degraded => "degraded",
            CollectorStatus::Failed => "failed",
        }
        .into(),
        detail: collector.detail.clone(),
    }
}

fn refresh_security_analysis(
    database: &Database,
    detection: &DetectionEngine,
    score: &ScoreEngine,
    mut coverage: Vec<ScoreCoverage>,
    issues: &mut Vec<CollectionIssue>,
) {
    let mut detection_status = ScoreCoverage {
        component: "detection-engine".into(),
        status: "healthy".into(),
        detail: "Factual event checkpoint is current".into(),
    };
    let mut batches = 0u8;
    loop {
        match detection.process_pending(database) {
            Ok(summary) => {
                batches = batches.saturating_add(1);
                if !summary.has_more || batches >= 8 {
                    if summary.has_more {
                        detection_status.status = "degraded".into();
                        detection_status.detail =
                            "Detection analysis backlog remains after the bounded refresh".into();
                    }
                    break;
                }
            }
            Err(message) => {
                detection_status.status = "failed".into();
                detection_status.detail =
                    "Detection analysis will retry from its checkpoint".into();
                issues.push(CollectionIssue {
                    component: "detection-engine".into(),
                    message,
                });
                break;
            }
        }
    }
    coverage.push(detection_status);
    if let Err(message) = score.calculate_and_persist(database, coverage) {
        issues.push(CollectionIssue {
            component: "security-score".into(),
            message,
        });
    }
}

#[tauri::command]
pub fn get_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "system-collector".into(),
            status: "available".into(),
            detail: "Windows system telemetry".into(),
        },
        Capability {
            id: "process-collector".into(),
            status: "available".into(),
            detail: "Native Windows process telemetry with cached executable metadata".into(),
        },
        Capability {
            id: "connection-collector".into(),
            status: "available".into(),
            detail: "IP Helper TCP and UDP tables with PID correlation".into(),
        },
        Capability {
            id: "service-collector".into(),
            status: "available".into(),
            detail: "Read-only Windows Service Control Manager telemetry".into(),
        },
        Capability {
            id: "network-collector".into(),
            status: "available".into(),
            detail: "Windows adapter telemetry".into(),
        },
        Capability {
            id: "sqlite".into(),
            status: "available".into(),
            detail: "Local snapshot and settings persistence".into(),
        },
        Capability {
            id: "behavioral-baseline".into(),
            status: "available".into(),
            detail: "Local factual behavioral baseline with versioned history".into(),
        },
        Capability {
            id: "security-events".into(),
            status: "available".into(),
            detail: "Local observation and change event foundation".into(),
        },
        Capability {
            id: "security-score".into(),
            status: "available".into(),
            detail: "Explainable local score derived from calibrated detections and coverage"
                .into(),
        },
        Capability {
            id: "detection-engine".into(),
            status: "available".into(),
            detail: format!(
                "{} versioned local rules over factual provenance",
                rules::registry().len()
            ),
        },
        Capability {
            id: "software-inventory".into(),
            status: "available".into(),
            detail: "Native read-only Windows installed software inventory".into(),
        },
        Capability {
            id: "vulnerability-repository".into(),
            status: "available".into(),
            detail: "Offline-first NVD and CISA KEV local repositories".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{get_system_locale, LANGUAGES, THEMES};

    #[test]
    fn only_documented_themes_are_accepted() {
        assert_eq!(THEMES.len(), 4);
        assert!(THEMES.contains(&"spectrum"));
        assert!(!THEMES.contains(&"neon-gamer"));
    }

    #[test]
    fn only_supported_interface_languages_are_accepted() {
        assert_eq!(LANGUAGES, ["pt-BR", "en"]);
        assert!(!LANGUAGES.contains(&"pt-PT"));
    }

    #[test]
    fn windows_user_locale_is_available_to_the_i18n_bootstrap() {
        let locale = get_system_locale().expect("Windows user locale should be readable");
        assert!(!locale.trim().is_empty());
        assert!(!locale.contains('\0'));
    }
}
