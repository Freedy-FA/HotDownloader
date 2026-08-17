mod commands;
mod download;
mod events;
mod storage;
mod utils;

use download::engine::DownloadEngine;
use storage::store_wrapper;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Debug)  // 可调整为 Info 或 Warn
            .build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_android_fs::init())
        .setup(|app| {
            let engine = DownloadEngine::new(app.handle().clone());
            let max_concurrent = match store_wrapper::load_string(app.handle(), "settings") {
                Ok(json) => {
                    serde_json::from_str::<serde_json::Value>(&json)
                        .ok()
                        .and_then(|v| v.get("maxConcurrent")?.as_u64())
                        .map(|n| n as u32)
                        .unwrap_or(3)
                }
                Err(_) => 3,
            };
            engine.set_concurrency(max_concurrent);
            commands::settings::restore_session(app.handle());
            app.manage(engine.clone());

            let engine_clone = engine;
            tauri::async_runtime::spawn(async move {
                engine_clone.run_scheduler().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::history::load_history,
            commands::history::save_history,
            commands::tasks::load_tasks,
            commands::tasks::save_tasks,
            commands::tasks::add_download_task,
            commands::tasks::enqueue_task,
            commands::tasks::pause_task,
            commands::tasks::resume_task,
            commands::tasks::cancel_task,
            commands::tasks::remove_task,
            commands::tasks::set_max_concurrent,
            commands::file_ops::get_default_download_dir,
            commands::file_ops::create_directory,
            commands::file_ops::open_file_location,
            commands::file_ops::pick_saf_folder,
            commands::file_ops::delete_saf_file,
            commands::api::search_songs,
            commands::api::fetch_download_link,
            commands::api::fetch_hot_keywords,
            commands::api::fetch_suggestions,
            commands::api::fetch_playlist_songs,
            commands::api::fetch_lyrics,
            commands::api::check_update,
            commands::login::start_qq_login,
            commands::login::capture_qq_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
