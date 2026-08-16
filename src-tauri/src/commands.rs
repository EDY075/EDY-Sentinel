use crate::{
    collectors,
    models::{Capability, DatabaseStatus, SystemOverview, ThemeInput},
    persistence::Database,
};
use tauri::State;

const THEMES: [&str; 4] = ["sentinel-blue", "cyber-green", "terminal", "spectrum"];

#[tauri::command]
pub async fn get_system_overview(database: State<'_, Database>) -> Result<SystemOverview, String> {
    let overview = tauri::async_runtime::spawn_blocking(collectors::collect_overview)
        .await
        .map_err(|_| "System collection task failed".to_string())??;
    database.save_snapshot(&overview)?;
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
pub fn get_capabilities() -> Vec<Capability> {
    vec![
        Capability {
            id: "system-collector".into(),
            status: "available".into(),
            detail: "Windows system telemetry".into(),
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
            id: "security-score".into(),
            status: "not_implemented".into(),
            detail: "Analysis engine not configured yet".into(),
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
