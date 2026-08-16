#[cfg(windows)]
mod baseline;
mod collectors;
mod commands;
#[cfg(windows)]
mod detection;
#[cfg(windows)]
mod detection_query;
#[cfg(windows)]
mod event_query;
#[cfg(windows)]
mod host_identity;
#[cfg(windows)]
mod inventory;
mod models;
mod persistence;
pub mod rules;
#[cfg(windows)]
mod score;
#[cfg(windows)]
mod telemetry;
#[cfg(windows)]
mod vulnerability;

use persistence::Database;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            let database = Database::open(app_data.join("sentinel.db"))?;
            #[cfg(windows)]
            vulnerability::recover_interrupted_syncs(&database)?;
            #[cfg(windows)]
            let detection = {
                let engine = detection::DetectionEngine;
                engine.initialize(&database, rules::registry())?;
                engine
            };
            app.manage(database);
            #[cfg(windows)]
            app.manage(Arc::new(vulnerability::VulnerabilitySyncManager::default()));
            #[cfg(windows)]
            app.manage(baseline::BaselineEngine::default());
            #[cfg(windows)]
            app.manage(detection);
            #[cfg(windows)]
            app.manage(score::ScoreEngine);
            #[cfg(windows)]
            app.manage(telemetry::TelemetryEngine::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_overview,
            commands::get_theme,
            commands::set_theme,
            commands::get_system_locale,
            commands::get_language,
            commands::set_language,
            commands::get_capabilities,
            commands::get_database_status,
            commands::get_live_telemetry,
            commands::get_baseline_summary,
            commands::get_security_events,
            commands::get_security_events_page,
            commands::get_security_event_history,
            commands::start_new_baseline,
            commands::reset_baseline,
            commands::complete_baseline_learning,
            commands::set_security_event_status,
            commands::get_detections_page,
            commands::get_detection_evidence,
            commands::set_detection_status,
            commands::get_detection_rules,
            commands::set_detection_rule_enabled,
            commands::get_security_score,
            commands::get_software_inventory,
            commands::refresh_software_inventory,
            commands::get_vulnerability_provider_status,
            commands::sync_vulnerability_provider,
            commands::cancel_vulnerability_sync,
        ])
        .run(tauri::generate_context!())
        .expect("EDY Sentinel failed to start");
}
