use crate::events;
use tauri::{AppHandle, Emitter};

pub fn emit_progress(
    app_handle: &AppHandle,
    task_id: &str,
    downloaded: u64,
    total: u64,
    speed: u64,
) {
    let payload = events::DownloadProgressPayload {
        task_id: task_id.to_string(),
        downloaded,
        total,
        speed,
    };
    let _ = app_handle.emit(events::DOWNLOAD_PROGRESS, payload);
}

pub fn emit_completed(
    app_handle: &AppHandle,
    task_id: &str,
    final_path: &str,
    saf_folder_uri: Option<String>,
    quality: &str,
    filename: &str,
    requested_quality: Option<&str>,
) {
    let payload = events::DownloadCompletedPayload {
        task_id: task_id.to_string(),
        final_path: final_path.to_string(),
        saf_folder_uri,
        quality: quality.to_string(),
        filename: filename.to_string(),
        requested_quality: requested_quality.map(|s| s.to_string()),
    };
    let _ = app_handle.emit(events::DOWNLOAD_COMPLETED, payload);
}

pub fn emit_error(app_handle: &AppHandle, task_id: &str, error_msg: &str) {
    let payload = events::DownloadErrorPayload {
        task_id: task_id.to_string(),
        error_msg: error_msg.to_string(),
    };
    let _ = app_handle.emit(events::DOWNLOAD_ERROR, payload);
}

pub fn emit_link_expired(app_handle: &AppHandle, task_id: &str, current_offset: u64) {
    let payload = events::DownloadLinkExpiredPayload {
        task_id: task_id.to_string(),
        current_offset,
    };
    let _ = app_handle.emit(events::DOWNLOAD_LINK_EXPIRED, payload);
}

pub fn emit_quality_changed(
    app_handle: &AppHandle,
    task_id: &str,
    requested_quality: &str,
    actual_quality: &str,
    filename: &str,
) {
    let payload = events::DownloadQualityChangedPayload {
        task_id: task_id.to_string(),
        requested_quality: requested_quality.to_string(),
        actual_quality: actual_quality.to_string(),
        filename: filename.to_string(),
    };
    let _ = app_handle.emit(events::DOWNLOAD_QUALITY_CHANGED, payload);
}
