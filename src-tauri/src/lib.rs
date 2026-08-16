mod collectors;
mod commands;
mod models;
mod persistence;

use persistence::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;
            app.manage(Database::open(app_data.join("sentinel.db"))?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_overview,
            commands::get_theme,
            commands::set_theme,
            commands::get_capabilities,
            commands::get_database_status,
        ])
        .run(tauri::generate_context!())
        .expect("EDY Sentinel failed to start");
}
