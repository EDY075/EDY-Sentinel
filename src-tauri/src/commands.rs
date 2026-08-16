use crate::{
    baseline::BaselineEngine,
    collectors,
    models::{
        BaselineActionInput, BaselineSummary, Capability, CollectionIssue, DatabaseStatus,
        LiveTelemetrySnapshot, SecurityEventRecord, SecurityEventStatusInput, SystemOverview,
        ThemeInput,
    },
    persistence::Database,
    telemetry::TelemetryEngine,
};
use tauri::State;

const THEMES: [&str; 4] = ["sentinel-blue", "cyber-green", "terminal", "spectrum"];

#[tauri::command]
pub async fn get_system_overview(
    database: State<'_, Database>,
    baseline: State<'_, BaselineEngine>,
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
) -> Result<LiveTelemetrySnapshot, String> {
    let engine = engine.inner().clone();
    let database = database.inner().clone();
    let baseline = baseline.inner().clone();
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
            status: "not_implemented".into(),
            detail: "Detection engine and explainable score are not implemented".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::THEMES;

    #[test]
    fn only_documented_themes_are_accepted() {
        assert_eq!(THEMES.len(), 4);
        assert!(THEMES.contains(&"spectrum"));
        assert!(!THEMES.contains(&"neon-gamer"));
    }
}
