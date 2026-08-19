use crate::storage::store_wrapper;
use tauri::{command, AppHandle};

#[command]
pub fn load_history(app: AppHandle) -> Result<String, String> {
    store_wrapper::load_string(&app, "history").map_err(|e| e.to_string())
}

#[command]
pub fn save_history(app: AppHandle, history_json: String) -> Result<(), String> {
    store_wrapper::save_string(&app, "history", &history_json).map_err(|e| e.to_string())
}
