use tauri::{command, AppHandle, Manager};
use crate::download::engine::DownloadEngine;
use crate::storage::store_wrapper;

#[command]
pub fn load_tasks(app: AppHandle) -> Result<String, String> {
    store_wrapper::load_string(&app, "tasks").map_err(|e| e.to_string())
}

#[command]
pub fn save_tasks(app: AppHandle, tasks_json: String) -> Result<(), String> {
    store_wrapper::save_string(&app, "tasks", &tasks_json).map_err(|e| e.to_string())
}

#[command]
pub async fn add_download_task(
    app: AppHandle,
    task_id: String,
    song_id: String,
    url: String,
    save_path: String,
    quality: String,
    filename: String,
    key: String,
    file_size: u64,
    song_title: String,
    artist: String,
    album: String,
) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>();
    engine
        .add_task(
            task_id, song_id, url, save_path, quality, filename, key, file_size, song_title,
            artist, album,
        )
        .await;
    Ok(())
}

#[command]
pub async fn enqueue_task(app: AppHandle, task_id: String, offset: u64) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>().clone();
    engine.enqueue_task(&task_id, offset).await;
    Ok(())
}

#[command]
pub async fn pause_task(app: AppHandle, task_id: String) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>().clone();
    engine.pause(&task_id).await;
    Ok(())
}

#[command]
pub async fn resume_task(app: AppHandle, task_id: String) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>().clone();
    engine.resume(&task_id).await;
    Ok(())
}

#[command]
pub async fn cancel_task(app: AppHandle, task_id: String, delete_file: bool) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>().clone();
    engine.cancel(&task_id, delete_file).await;
    Ok(())
}

#[command]
pub async fn remove_task(app: AppHandle, task_id: String, delete_file: bool) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>().clone();
    engine.remove(&task_id, delete_file).await
}

#[command]
pub fn set_max_concurrent(app: AppHandle, max: u32) -> Result<(), String> {
    let engine = app.state::<DownloadEngine>();
    engine.set_concurrency(max);
    Ok(())
}
