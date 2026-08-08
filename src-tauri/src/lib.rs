mod commands;
mod download;
mod storage;
mod utils;
mod events;

use download::engine::DownloadEngine;
use storage::store_wrapper;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
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
            commands::tasks::update_task_url,
            commands::tasks::pause_task,
            commands::tasks::resume_task,
            commands::tasks::cancel_task,
            commands::tasks::set_max_concurrent,
            commands::file_ops::get_default_download_dir,
            commands::file_ops::create_directory,
            commands::file_ops::open_file_location,
            commands::api::search_songs,
            commands::api::fetch_download_link,
            commands::api::fetch_hot_keywords,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}