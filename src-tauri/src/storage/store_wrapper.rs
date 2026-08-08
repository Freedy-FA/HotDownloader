use tauri::AppHandle;
use tauri::Manager;

/// 从 Tauri Store 加载字符串数据
pub fn load_string(app: &AppHandle, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let store = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .ok_or("Store not initialized")?;
    let value = store
        .get(key.to_string())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    Ok(value)
}

/// 保存字符串数据到 Tauri Store
pub fn save_string(
    app: &AppHandle,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .ok_or("Store not initialized")?;
    store.set(
        key.to_string(),
        serde_json::Value::String(value.to_string()),
    );
    store.save()?;
    Ok(())
}