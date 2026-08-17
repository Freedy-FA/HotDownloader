use crate::storage::store_wrapper;
use crate::utils::auth;
use tauri::{command, AppHandle};

fn apply_session_from_settings_json(settings_json: &str) {
    let cookie = serde_json::from_str::<serde_json::Value>(settings_json)
        .ok()
        .and_then(|v| {
            v.get("qqCookie")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();
    auth::set_session_from_cookie(&cookie);
}

/// 启动时从本地设置恢复 QQ 音乐会话。
pub fn restore_session(app: &AppHandle) {
    if let Ok(json) = store_wrapper::load_string(app, "settings") {
        apply_session_from_settings_json(&json);
    }
}

#[command]
pub fn load_settings(app: AppHandle) -> Result<String, String> {
    store_wrapper::load_string(&app, "settings").map_err(|e| e.to_string())
}

#[command]
pub fn save_settings(app: AppHandle, settings_json: String) -> Result<(), String> {
    apply_session_from_settings_json(&settings_json);
    store_wrapper::save_string(&app, "settings", &settings_json).map_err(|e| e.to_string())
}