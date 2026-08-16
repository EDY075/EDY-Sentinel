#[cfg(windows)]
mod baseline;
mod collectors;
mod commands;
#[cfg(windows)]
mod event_query;
#[cfg(windows)]
mod host_identity;
mod models;
mod persistence;
pub mod rules;
#[cfg(windows)]
mod telemetry;

use persistence::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            app.manage(Database::open(app_data.join("sentinel.db"))?);
            #[cfg(windows)]
            app.manage(baseline::BaselineEngine::default());
            #[cfg(windows)]
            app.manage(telemetry::TelemetryEngine::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_overview,
            commands::get_theme,
            commands::set_theme,
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
        ])
        .run(tauri::generate_context!())
        .expect("EDY Sentinel failed to start");
}
