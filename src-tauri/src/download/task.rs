use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use tauri::AppHandle;

use super::engine::TaskController;
use super::progress;
use crate::utils::{crypto, filename};

/// 歌曲信息，用于生成文件名
#[derive(Clone)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
}

/// 单个任务的上下文信息
pub struct TaskContext {
    pub task_id: String,
    pub url: String,
    pub save_path: String,
    pub quality: String,
    pub key: String,
    pub file_size: u64,
    pub downloaded_offset: u64,
    pub app_handle: AppHandle,
    pub song_info: SongInfo,
}

/// 实际执行下载的函数
pub async fn download_task(
    ctx: TaskContext,
    controller: TaskController,
    app_handle: AppHandle,
) {
    // 1. 等待 URL 就绪（如果 url 还为空）
    if ctx.url.is_empty() {
        controller.url_ready.notified().await;
    }

    // 2. 构建最终保存路径
    let download_dir = {
        if !ctx.save_path.is_empty() {
            ctx.save_path.clone()
        } else {
            let (dir, template) = get_download_settings(&app_handle).await;
            let song = &ctx.song_info;
            let fname = filename::build_filename(&template, song);
            let full_path = Path::new(&dir).join(fname);
            full_path.to_string_lossy().to_string()
        }
    };

    // 3. 创建目录并验证
    let parent_dir = Path::new(&download_dir).parent().unwrap_or(Path::new("."));
    if !parent_dir.exists() {
        if let Err(_e) = fs::create_dir_all(parent_dir) {
            progress::emit_error(&app_handle, &ctx.task_id, "下载目录无法访问");
            return;
        }
    }

    // 4. 打开/创建文件（追加模式）
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&download_dir)
    {
        Ok(f) => f,
        Err(e) => {
            progress::emit_error(&app_handle, &ctx.task_id, &format!("文件创建失败: {}", e));
            return;
        }
    };

    // 如果续传且文件大小小于 offset，重置文件
    let mut downloaded = ctx.downloaded_offset;
    if downloaded > 0 {
        if let Ok(metadata) = file.metadata() {
            if metadata.len() < downloaded {
                if let Err(e) = file.set_len(0) {
                    progress::emit_error(&app_handle, &ctx.task_id, &format!("文件重置失败: {}", e));
                    return;
                }
                downloaded = 0;
            }
        }
    }

    let total = ctx.file_size;

    // 5. 解密上下文
    let decrypt_ctx = crypto::init_decryption(&ctx.key, !ctx.key.is_empty());

    // 6. 下载循环
    'download: loop {
        if controller.cancel_token.is_cancelled() {
            break 'download;
        }

        while controller.pause_flag.load(Ordering::SeqCst) {
            controller.resume_notify.notified().await;
            if controller.cancel_token.is_cancelled() {
                break 'download;
            }
        }

        let client = reqwest::Client::new();
        let mut request = client.get(&ctx.url);
        if downloaded > 0 {
            request = request.header(RANGE, format!("bytes={}-", downloaded));
        }

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                progress::emit_error(&app_handle, &ctx.task_id, &format!("网络错误: {}", e));
                break 'download;
            }
        };

        let status = response.status();
        if status == StatusCode::RANGE_NOT_SATISFIABLE {
            progress::emit_completed(&app_handle, &ctx.task_id, &download_dir);
            break 'download;
        }

        if status.is_client_error() || status.is_server_error() {
            let error_msg = if status == StatusCode::FORBIDDEN
                || status == StatusCode::GONE
                || status == StatusCode::NOT_FOUND
            {
                progress::emit_link_expired(&app_handle, &ctx.task_id, downloaded);
                "链接过期"
            } else {
                "下载失败"
            };
            progress::emit_error(&app_handle, &ctx.task_id, error_msg);
            break 'download;
        }

        let mut stream = response.bytes_stream();
        let mut last_report = Instant::now();
        let mut last_downloaded = downloaded;

        while let Some(chunk_result) = stream.next().await {
            if controller.cancel_token.is_cancelled() {
                break 'download;
            }
            while controller.pause_flag.load(Ordering::SeqCst) {
                controller.resume_notify.notified().await;
                if controller.cancel_token.is_cancelled() {
                    break 'download;
                }
            }

            let chunk = match chunk_result {
                Ok(bytes) => bytes,
                Err(e) => {
                    progress::emit_error(&app_handle, &ctx.task_id, &format!("读取流错误: {}", e));
                    break 'download;
                }
            };

            // 转换为可变的 Vec<u8>
            let mut chunk_data = chunk.to_vec();
            let chunk_len = chunk_data.len() as u64;

            // 解密
            crypto::decrypt_chunk(&decrypt_ctx, &mut chunk_data, chunk_len, downloaded);

            // 写入文件
            if let Err(e) = file.write_all(&chunk_data) {
                progress::emit_error(&app_handle, &ctx.task_id, &format!("写入文件错误: {}", e));
                break 'download;
            }

            downloaded += chunk_len;

            let now = Instant::now();
            let elapsed = now - last_report;
            if elapsed >= Duration::from_millis(500) {
                let speed = if elapsed.as_secs_f64() > 0.0 {
                    ((downloaded - last_downloaded) as f64 / elapsed.as_secs_f64()) as u64
                } else {
                    0
                };
                progress::emit_progress(&app_handle, &ctx.task_id, downloaded, total, speed);
                last_report = now;
                last_downloaded = downloaded;
            }

            if total > 0 && downloaded >= total {
                progress::emit_completed(&app_handle, &ctx.task_id, &download_dir);
                break 'download;
            }
        }

        if !controller.cancel_token.is_cancelled()
            && !controller.pause_flag.load(Ordering::SeqCst)
        {
            progress::emit_completed(&app_handle, &ctx.task_id, &download_dir);
        }
        break 'download;
    }

    let _ = file.sync_all();
}

async fn get_download_settings(app_handle: &AppHandle) -> (String, String) {
    use crate::storage::store_wrapper;
    let default_dir = dirs::download_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .to_string_lossy()
        .to_string();
    let default_template = "{song} - {artist}".to_string();

    let settings_json = store_wrapper::load_string(app_handle, "settings").unwrap_or_default();
    let settings: serde_json::Value =
        serde_json::from_str(&settings_json).unwrap_or(serde_json::json!({}));
    let dir = settings
        .get("downloadDir")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_dir)
        .to_string();
    let template = settings
        .get("namingTemplate")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_template)
        .to_string();
    (dir, template)
}