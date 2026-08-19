use crate::storage::store_wrapper;
use tauri::{command, AppHandle};

#[command]
pub fn load_settings(app: AppHandle) -> Result<String, String> {
    store_wrapper::load_string(&app, "settings").map_err(|e| e.to_string())
}

#[command]
pub fn save_settings(app: AppHandle, settings_json: String) -> Result<(), String> {
    store_wrapper::save_string(&app, "settings", &settings_json).map_err(|e| e.to_string())
}
