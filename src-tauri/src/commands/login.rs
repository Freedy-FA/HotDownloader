use crate::storage::store_wrapper;
use crate::utils::auth;
use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use url::Url;

const LOGIN_WINDOW_LABEL: &str = "qq-login";
const LOGIN_URL: &str = "https://y.qq.com/";
const COOKIE_URLS: &[&str] = &[
    "https://y.qq.com/",
    "https://u.y.qq.com/",
    "https://c.y.qq.com/",
    "https://qq.com/",
];

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QqLoginCaptured {
    pub cookie: String,
    pub uin: String,
}

fn persist_cookie(app: &AppHandle, cookie: &str) {
    auth::set_session_from_cookie(cookie);
    let mut settings = store_wrapper::load_string(app, "settings")
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    if !settings.is_object() {
        settings = serde_json::json!({});
    }
    settings["qqCookie"] = serde_json::Value::String(cookie.to_string());
    let _ = store_wrapper::save_string(app, "settings", &settings.to_string());
}

fn collect_cookie_pairs(window: &tauri::WebviewWindow) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for raw in COOKIE_URLS {
        let Ok(url) = Url::parse(raw) else {
            continue;
        };
        if let Ok(cookies) = window.cookies_for_url(url) {
            for cookie in cookies {
                pairs.push((cookie.name().to_string(), cookie.value().to_string()));
            }
        }
    }
    if pairs.is_empty() {
        if let Ok(cookies) = window.cookies() {
            for cookie in cookies {
                pairs.push((cookie.name().to_string(), cookie.value().to_string()));
            }
        }
    }
    pairs
}

fn try_capture_from_window(window: &tauri::WebviewWindow) -> Option<QqLoginCaptured> {
    let header = auth::format_cookie_header(&collect_cookie_pairs(window));
    let session = auth::parse_qq_session(&header)?;
    Some(QqLoginCaptured {
        cookie: session.cookie,
        uin: session.uin,
    })
}

fn finish_login(app: &AppHandle, captured: QqLoginCaptured) {
    persist_cookie(app, &captured.cookie);
    let _ = app.emit("qq-login-success", captured);
    if let Some(window) = app.get_webview_window(LOGIN_WINDOW_LABEL) {
        let _ = window.close();
    }
}

#[tauri::command]
pub async fn start_qq_login(app: AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(LOGIN_WINDOW_LABEL) {
        let _ = existing.set_focus();
        return Ok(());
    }

    let login_url = Url::parse(LOGIN_URL).map_err(|e| e.to_string())?;
    let app_for_load = app.clone();
    let builder = WebviewWindowBuilder::new(
        &app,
        LOGIN_WINDOW_LABEL,
        WebviewUrl::External(login_url),
    )
    .title("登录 QQ 音乐")
    .inner_size(980.0, 720.0)
    .on_page_load(move |window, _payload| {
        let app = app_for_load.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(400)).await;
            if let Some(captured) = try_capture_from_window(&window) {
                finish_login(&app, captured);
            }
        });
    });

    #[cfg(desktop)]
    let builder = builder.center().focused(true);

    builder.build().map_err(|e| format!("打开登录窗口失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn capture_qq_login(app: AppHandle) -> Result<QqLoginCaptured, String> {
    let window = app
        .get_webview_window(LOGIN_WINDOW_LABEL)
        .ok_or_else(|| "登录窗口未打开".to_string())?;
    let captured = try_capture_from_window(&window)
        .ok_or_else(|| "尚未检测到登录，请在打开的窗口中完成 QQ 音乐登录后再试".to_string())?;
    finish_login(&app, captured.clone());
    Ok(captured)
}
