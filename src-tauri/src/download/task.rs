use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use tauri::AppHandle;
use tauri::Manager; // 提供 try_state 方法

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
    pub save_path: String, // 最终文件路径
    pub quality: String,
    pub key: String,
    pub file_size: u64,
    pub downloaded_offset: u64,
    pub app_handle: AppHandle,
    pub song_info: SongInfo,
    pub quality_filename: String,
}

/// 实际执行下载的函数
pub async fn download_task(ctx: TaskContext, controller: TaskController, app_handle: AppHandle) {
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
            // 从品质文件名中提取扩展名，若无法提取则回退为 "flac"
            let raw_ext = Path::new(&ctx.quality_filename)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("flac");
            // 映射解密后的扩展名
            let ext = map_decrypted_extension(raw_ext);
            let full_path = Path::new(&dir).join(format!("{}.{}", fname, ext));
            full_path.to_string_lossy().to_string()
        }
    };

    log::info!("任务 {} 开始下载，文件路径: {}", ctx.task_id, download_dir);

    // 3. 创建目录并验证
    let parent_dir = Path::new(&download_dir).parent().unwrap_or(Path::new("."));
    if !parent_dir.exists() {
        if let Err(e) = fs::create_dir_all(parent_dir) {
            log::error!("创建下载目录失败: {}", e);
            progress::emit_error(&app_handle, &ctx.task_id, "下载目录无法访问");
            return;
        }
    }

    // 4. 根据 offset 决定打开模式：新任务覆盖，续传任务追加
    let mut downloaded = ctx.downloaded_offset;

    let mut file = if downloaded == 0 {
        // 全新下载，覆盖已有文件
        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&download_dir)
        {
            Ok(f) => f,
            Err(e) => {
                log::error!("文件创建失败: {}", e);
                progress::emit_error(&app_handle, &ctx.task_id, &format!("文件创建失败: {}", e));
                return;
            }
        }
    } else {
        // 续传任务，先以追加模式打开
        let f = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&download_dir)
        {
            Ok(f) => f,
            Err(e) => {
                log::error!("文件打开失败: {}", e);
                progress::emit_error(&app_handle, &ctx.task_id, &format!("文件打开失败: {}", e));
                return;
            }
        };

        // 校验文件大小：如果文件长度小于期望的偏移，说明文件异常，重置下载
        if let Ok(metadata) = f.metadata() {
            if metadata.len() < downloaded {
                // 文件被截断或损坏，清空文件并从头下载
                drop(f); // 先关闭文件，避免占用
                let mut new_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&download_dir)
                    .map_err(|e| {
                        log::error!("文件重置失败: {}", e);
                        progress::emit_error(
                            &app_handle,
                            &ctx.task_id,
                            &format!("文件重置失败: {}", e),
                        );
                    })
                    .ok();
                if new_file.is_none() {
                    return;
                }
                downloaded = 0;
                new_file.unwrap()
            } else {
                f
            }
        } else {
            // 无法获取元数据，保守起见改为从头下载
            drop(f);
            let mut new_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&download_dir)
                .map_err(|e| {
                    log::error!("文件重置失败: {}", e);
                    progress::emit_error(
                        &app_handle,
                        &ctx.task_id,
                        &format!("文件重置失败: {}", e),
                    );
                })
                .ok();
            if new_file.is_none() {
                return;
            }
            downloaded = 0;
            new_file.unwrap()
        }
    };

    // 5. 解密上下文
    let decrypt_ctx = crypto::init_decryption(&ctx.key, !ctx.key.is_empty());

    // 6. 下载循环
    'download: loop {
        // 检查取消
        if controller.cancel_token.is_cancelled() {
            break 'download;
        }

        // 初始暂停等待（任务刚创建时可能处于暂停状态）
        while controller.pause_flag.load(Ordering::SeqCst) {
            controller.resume_notify.notified().await;
            if controller.cancel_token.is_cancelled() {
                break 'download;
            }
        }

        let client = reqwest::Client::builder()
            .user_agent("HotDownloader/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut request = client.get(&ctx.url).header("Referer", "https://y.qq.com");

        if downloaded > 0 {
            request = request.header(RANGE, format!("bytes={}-", downloaded));
        }

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                log::error!("网络错误: {}", e);
                progress::emit_error(&app_handle, &ctx.task_id, &format!("网络错误: {}", e));
                break 'download;
            }
        };

        // 从响应头获取真实文件总大小
        let total = {
            if let Some(content_range) = response.headers().get("content-range") {
                content_range
                    .to_str()
                    .ok()
                    .and_then(|s| s.split('/').last().and_then(|n| n.parse::<u64>().ok()))
                    .unwrap_or(0)
            } else {
                response
                    .headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
            }
        };
        let total = if total > 0 { total } else { ctx.file_size };

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
            log::error!("下载错误: {}", error_msg);
            progress::emit_error(&app_handle, &ctx.task_id, error_msg);
            break 'download;
        }

        let mut stream = response.bytes_stream();
        let mut last_report = Instant::now();
        let mut last_downloaded = downloaded;

        // 内部流读取循环
        loop {
            // 检查取消
            if controller.cancel_token.is_cancelled() {
                break 'download;
            }

            // 检查暂停：如果暂停，跳出内部循环，回到外层重新请求
            if controller.pause_flag.load(Ordering::SeqCst) {
                break;
            }

            let chunk_result = stream.next().await;
            let chunk = match chunk_result {
                Some(Ok(bytes)) => bytes,
                Some(Err(e)) => {
                    log::error!("读取流错误: {}", e);
                    progress::emit_error(&app_handle, &ctx.task_id, &format!("读取流错误: {}", e));
                    break 'download;
                }
                None => {
                    // 流正常结束，跳出内部循环去发送完成事件
                    break;
                }
            };

            // 转换为可变的 Vec<u8>
            let mut chunk_data = chunk.to_vec();
            let chunk_len = chunk_data.len() as u64;

            // 解密
            crypto::decrypt_chunk(&decrypt_ctx, &mut chunk_data, chunk_len, downloaded);

            // 写入文件
            if let Err(e) = file.write_all(&chunk_data) {
                log::error!("写入文件错误: {}", e);
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
                log::info!("下载完成: {}", download_dir);
                progress::emit_completed(&app_handle, &ctx.task_id, &download_dir);
                break 'download;
            }
        }

        // 跳出内部循环后的处理
        if controller.pause_flag.load(Ordering::SeqCst) {
            // 因暂停跳出，等待恢复通知
            loop {
                controller.resume_notify.notified().await;
                if !controller.pause_flag.load(Ordering::SeqCst) {
                    break;
                }
                if controller.cancel_token.is_cancelled() {
                    break 'download;
                }
            }
            // 恢复后继续外层循环（将重新发送请求）
            continue 'download;
        }

        // 不是暂停（流结束），发送完成事件
        if !controller.cancel_token.is_cancelled() {
            progress::emit_completed(&app_handle, &ctx.task_id, &download_dir);
        }
        break 'download;
    }

    // 显式关闭文件句柄，释放资源
    drop(file);

    // 如果任务被取消且用户要求删除文件，执行删除
    if controller.cancel_token.is_cancelled()
        && controller.delete_file_on_cancel.load(Ordering::SeqCst)
    {
        log::info!("取消任务，正在删除文件: {}", download_dir);
        if let Err(e) = fs::remove_file(&download_dir) {
            log::error!("删除文件失败: {}", e);
        } else {
            log::info!("文件已成功删除: {}", download_dir);
        }
    }
}

/// 获取下载目录（绝对路径）及文件命名模板
async fn get_download_settings(app_handle: &AppHandle) -> (String, String) {
    use crate::storage::store_wrapper;

    let default_dir = get_default_download_dir();
    let default_template = "{song} - {artist}".to_string();

    let settings_json = store_wrapper::load_string(app_handle, "settings").unwrap_or_default();
    let settings: serde_json::Value =
        serde_json::from_str(&settings_json).unwrap_or(serde_json::json!({}));

    let dir = settings
        .get("downloadDir")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default_dir.clone());

    // 确保目录路径是绝对路径，否则回退到默认下载目录
    let dir = if Path::new(&dir).is_absolute() {
        dir
    } else {
        log::warn!(
            "下载目录不是绝对路径，已回退为默认下载目录: {}",
            default_dir
        );
        default_dir
    };

    let template = settings
        .get("namingTemplate")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_template)
        .to_string();

    (dir, template)
}

/// 获取系统默认下载目录，失败时使用临时目录（确保绝对路径）
fn get_default_download_dir() -> String {
    if let Some(d) = dirs::download_dir() {
        return d.to_string_lossy().to_string();
    }
    if let Some(home) = dirs::home_dir() {
        let fallback = home.join("Downloads");
        return fallback.to_string_lossy().to_string();
    }
    // 最终回退到临时目录
    std::env::temp_dir().to_string_lossy().to_string()
}

/// 将加密文件扩展名映射为解密后的真实扩展名
fn map_decrypted_extension(ext: &str) -> &str {
    match ext {
        "mgg" => "ogg",
        "mflac" => "flac",
        // 未知则保持原样
        _ => ext,
    }
}
