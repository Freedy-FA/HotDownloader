use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// 从默认 data.json 存储加载字符串
pub fn load_string(app: &AppHandle, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let store = app.store("data.json")?;
    let value = store
        .get(key.to_string())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    Ok(value)
}

/// 保存字符串到默认 data.json 存储
pub fn save_string(
    app: &AppHandle,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = app.store("data.json")?;
    store.set(
        key.to_string(),
        serde_json::Value::String(value.to_string()),
    );
    store.save()?;
    Ok(())
}
