use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use once_cell::sync::Lazy;
use reqwest::header::{CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use tauri::AppHandle;
use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, FsUri};
use tokio::sync::Mutex;

use super::engine::TaskController;
use super::progress;
use crate::commands::api::{self}; // 获取下载链接
use crate::utils::{crypto, filename};

/// 文件写入缓冲区容量（64 KB）
const FILE_BUFFER_CAPACITY: usize = 64 * 1024;

/// 下载专用 HTTP 客户端：不设总超时，避免大文件下载中断；设置读取超时 5 分钟
static DOWNLOAD_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent("HotDownloader/1.0")
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(300)) // 5 分钟读取超时
        .build()
        .expect("Failed to create download HTTP client")
});

/// 歌曲信息，用于生成文件名
#[derive(Clone)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub quality: String,
    pub cover_url: String,
}

/// 单个任务的上下文信息
#[derive(Clone)]
pub struct TaskContext {
    pub task_id: String,
    pub song_id: String,
    pub url: String,
    pub save_path: String, // 最终文件路径
    #[allow(dead_code)] // 抑制未使用警告，保留备用
    pub quality: String,
    #[allow(dead_code)] // 抑制未使用警告，保留备用
    pub key: String,
    pub file_size: u64,
    pub downloaded_offset: u64,
    #[allow(dead_code)] // 抑制未使用警告，保留备用
    pub app_handle: AppHandle,
    pub song_info: SongInfo,
    pub quality_filename: String,
    pub final_path: Arc<Mutex<Option<String>>>, // 与控制器共享的文件路径
}

/// 重试获取下载链接。
/// 仅对瞬时网络错误重试；平台拒绝（104003 / 无法获取下载链接）立即返回，
/// 由 `get_download_link` 内部走加密回退。
/// 返回 (url, ekey, 实际品质文件名)。
async fn fetch_download_link_with_retry(
    song_id: &str,
    filename: &str,
    task_id: &str,
) -> Result<(String, String, String), String> {
    let mut last_err = String::new();
    for attempt in 0..3 {
        match api::get_download_link(song_id, filename).await {
            Ok(link) => return Ok(link),
            Err(e) => {
                last_err = e;
                log::warn!(
                    "任务 {} 获取下载链接失败 (尝试 {}/3): {}",
                    task_id,
                    attempt + 1,
                    last_err
                );
                if !crate::utils::quality::is_retryable_link_error(&last_err) {
                    return Err(last_err);
                }
                if attempt < 2 {
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                }
            }
        }
    }
    Err(last_err)
}

/// 判断错误是否属于可重试的网络类错误
fn is_retryable_network_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || (err.is_request() && !err.is_body())
}

/// 等待暂停恢复，返回 true 表示任务已被取消，应退出下载循环
async fn wait_for_resume_async(controller: &TaskController) -> bool {
    loop {
        controller.resume_notify.notified().await;
        if !controller.pause_flag.load(Ordering::SeqCst) {
            return false; // 恢复
        }
        if controller.cancel_token.is_cancelled() {
            return true; // 取消
        }
    }
}

/// 按实际品质文件名计算最终保存路径。
/// 降级可能把 `.mp3` 换成 `.mgg`，必须用实际文件名决定扩展名。
fn resolve_save_target(
    explicit_path: &str,
    settings_dir: &str,
    template: &str,
    saf_uri: &Option<String>,
    song_info: &SongInfo,
    quality_filename: &str,
) -> (bool, String, Option<String>) {
    if !explicit_path.is_empty() {
        return (false, explicit_path.to_string(), None);
    }

    let fname = filename::build_filename(template, song_info);
    let raw_ext = Path::new(quality_filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("flac");
    let ext = map_decrypted_extension(raw_ext);

    if settings_dir == "saf://" && cfg!(target_os = "android") && saf_uri.is_some() {
        (true, format!("{}.{}", fname, ext), saf_uri.clone())
    } else {
        let full_path = Path::new(settings_dir).join(format!("{}.{}", fname, ext));
        (false, full_path.to_string_lossy().to_string(), None)
    }
}

/// 实际执行下载的函数
pub async fn download_task(ctx: TaskContext, controller: TaskController, app_handle: AppHandle) {
    // 1. 先读设置；真正的保存路径等拿到实际品质文件名后再建
    let (settings_dir, template, saf_uri) = if ctx.save_path.is_empty() {
        get_download_settings(&app_handle).await
    } else {
        (String::new(), String::new(), None)
    };

    let mut is_saf = false;
    let mut download_dir = String::new();
    let mut saf_folder_uri: Option<String> = None;
    let mut path_ready = false;
    let mut actual_quality_filename = ctx.quality_filename.clone();

    // 2. 初始化已下载偏移量
    let mut downloaded = ctx.downloaded_offset;

    // 3. 链接与解密密钥（每次循环可能重新获取）
    let mut url = String::new();
    let mut key = String::new();

    // 4. 文件句柄（使用 BufWriter 提升写入性能）
    let mut file: Option<BufWriter<fs::File>> = None;

    let mut saf_file_uri: Option<String> = None;

    // 流错误重试计数器（防止无限重试）
    let mut stream_retries: u32 = 0;
    const MAX_STREAM_RETRIES: u32 = 2;

    // 下载循环
    'download: loop {
        // 检查取消
        if controller.cancel_token.is_cancelled() {
            break 'download;
        }

        // 初始暂停等待（使用统一的辅助函数）
        while controller.pause_flag.load(Ordering::SeqCst) {
            if wait_for_resume_async(&controller).await {
                break 'download; // 任务被取消
            }
        }

        // 如果没有有效链接，实时获取（首次进入或暂停恢复后）
        if url.is_empty() {
            match fetch_download_link_with_retry(&ctx.song_id, &ctx.quality_filename, &ctx.task_id)
                .await
            {
                Ok((new_url, new_key, used_filename)) => {
                    url = new_url;
                    key = new_key;
                    if used_filename != actual_quality_filename {
                        let requested = crate::utils::quality::quality_label_from_filename(
                            &actual_quality_filename,
                        );
                        let actual =
                            crate::utils::quality::quality_label_from_filename(&used_filename);
                        log::info!(
                            "任务 {} 品质回退: {} -> {}",
                            ctx.task_id,
                            requested,
                            actual
                        );
                        progress::emit_quality_changed(
                            &app_handle,
                            &ctx.task_id,
                            &requested,
                            &actual,
                            &used_filename,
                        );
                        actual_quality_filename = used_filename;
                        file = None;
                        downloaded = 0;
                        path_ready = false;
                    }
                    log::info!("任务 {} 获取到新下载链接", ctx.task_id);
                }
                Err(e) => {
                    log::error!("任务 {} 最终获取下载链接失败: {}", ctx.task_id, e);
                    let user_msg = if crate::utils::quality::is_retryable_link_error(&e) {
                        "网络错误，请稍后重试".to_string()
                    } else {
                        e
                    };
                    progress::emit_error(&app_handle, &ctx.task_id, &user_msg);
                    break 'download;
                }
            }
        }

        if !path_ready {
            let (saf, dir, uri) = resolve_save_target(
                &ctx.save_path,
                &settings_dir,
                &template,
                &saf_uri,
                &ctx.song_info,
                &actual_quality_filename,
            );
            is_saf = saf;
            download_dir = dir;
            saf_folder_uri = uri;
            path_ready = true;
            log::info!("任务 {} 开始下载，文件路径: {}", ctx.task_id, download_dir);

            // SAF 模式下记录父目录与音频文件名，供删除时清理同名 .lrc
            if is_saf {
                if let Some(folder) = &saf_folder_uri {
                    *controller.saf_lrc_meta.lock().await =
                        Some((folder.clone(), download_dir.clone()));
                }
            }

            if !is_saf {
                let parent_dir = Path::new(&download_dir).parent().unwrap_or(Path::new("."));
                if !parent_dir.exists() {
                    if let Err(e) = fs::create_dir_all(parent_dir) {
                        log::error!("创建下载目录失败: {}", e);
                        progress::emit_error(&app_handle, &ctx.task_id, "下载目录无法访问");
                        break 'download;
                    }
                }
            }
        }

        // 根据是否需要解密创建解密上下文
        let need_decrypt = {
            let ext = Path::new(&actual_quality_filename)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            ext == "mgg" || ext == "mflac"
        };
        let decrypt_ctx = if need_decrypt && !key.is_empty() {
            crypto::init_decryption(&key, true)
        } else {
            crypto::init_decryption("", false)
        };

        // 打开/续传文件
        if file.is_none() {
            let f = if is_saf {
                // SAF 模式
                let api = app_handle.android_fs();

                // 解析父目录 FsUri（包含 document_top_tree_uri）
                let parent_uri = match FsUri::from_json_str(saf_folder_uri.as_ref().unwrap()) {
                    Ok(uri) => uri,
                    Err(e) => {
                        log::error!("解析 SAF 文件夹 URI 失败: {}", e);
                        progress::emit_error(&app_handle, &ctx.task_id, "SAF 配置错误");
                        break 'download;
                    }
                };

                // download_dir 此时是文件名
                let file_path = std::path::Path::new(&download_dir);

                // 尝试解析已存在的文件
                let existing_file_uri = api.resolve_file_uri(&parent_uri, file_path).ok();

                match existing_file_uri {
                    Some(file_uri) => {
                        // 文件已存在，记录最终文件 URI
                        saf_file_uri = Some(file_uri.uri.clone());

                        if downloaded > 0 {
                            // 续传模式：打开文件并校验大小后 seek 到偏移量
                            match api.open_file(&file_uri, FileAccessMode::ReadWrite) {
                                Ok(mut f) => {
                                    use std::io::Seek;

                                    // 校验文件大小：如果文件长度小于期望的偏移，说明文件异常，重置下载
                                    let should_reset = match f.metadata() {
                                        Ok(meta) => meta.len() < downloaded,
                                        Err(_) => true, // 无法获取元数据，保守重置
                                    };

                                    if should_reset {
                                        log::warn!(
                                            "任务 {} SAF 文件大小异常，重置下载（期望偏移 {}，实际大小 {}）",
                                            ctx.task_id,
                                            downloaded,
                                            f.metadata().map(|m| m.len()).unwrap_or(0)
                                        );
                                        // 清空文件并从头下载
                                        if let Err(e) = f.set_len(0) {
                                            log::error!("SAF 文件截断失败: {}", e);
                                            progress::emit_error(
                                                &app_handle,
                                                &ctx.task_id,
                                                "文件异常，请重试",
                                            );
                                            break 'download;
                                        }
                                        if let Err(e) = f.seek(std::io::SeekFrom::Start(0)) {
                                            log::error!("SAF 文件 seek 失败: {}", e);
                                            progress::emit_error(
                                                &app_handle,
                                                &ctx.task_id,
                                                "文件定位失败",
                                            );
                                            break 'download;
                                        }
                                        downloaded = 0;
                                    } else {
                                        // 文件大小正常，seek 到续传位置
                                        if let Err(e) = f.seek(std::io::SeekFrom::Start(downloaded))
                                        {
                                            log::error!("SAF 文件 seek 失败: {}", e);
                                            progress::emit_error(
                                                &app_handle,
                                                &ctx.task_id,
                                                "文件定位失败",
                                            );
                                            break 'download;
                                        }
                                    }
                                    Some(f)
                                }
                                Err(e) => {
                                    log::error!("SAF 打开文件失败: {}", e);
                                    progress::emit_error(&app_handle, &ctx.task_id, "无法打开文件");
                                    break 'download;
                                }
                            }
                        } else {
                            // 从头开始：截断文件
                            match api.open_file_writable(&file_uri) {
                                Ok(f) => Some(f),
                                Err(e) => {
                                    log::error!("SAF 打开文件失败: {}", e);
                                    progress::emit_error(&app_handle, &ctx.task_id, "无法打开文件");
                                    break 'download;
                                }
                            }
                        }
                    }
                    None => {
                        // 文件不存在，创建新文件
                        match api.create_new_file(&parent_uri, file_path, None) {
                            Ok(file_uri) => {
                                saf_file_uri = Some(file_uri.uri.clone());
                                // 新文件，重置偏移量
                                if downloaded > 0 {
                                    downloaded = 0;
                                }
                                match api.open_file_writable(&file_uri) {
                                    Ok(f) => Some(f),
                                    Err(e) => {
                                        log::error!("SAF 打开新文件失败: {}", e);
                                        progress::emit_error(
                                            &app_handle,
                                            &ctx.task_id,
                                            "无法打开文件",
                                        );
                                        break 'download;
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("SAF 创建文件失败: {}", e);
                                progress::emit_error(&app_handle, &ctx.task_id, "无法创建文件");
                                break 'download;
                            }
                        }
                    }
                }
            } else {
                // 普通模式：原有逻辑
                if downloaded == 0 {
                    match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&download_dir)
                    {
                        Ok(f) => Some(f),
                        Err(e) => {
                            log::error!("文件创建失败: {}", e);
                            progress::emit_error(
                                &app_handle,
                                &ctx.task_id,
                                "文件创建失败，请检查磁盘空间",
                            );
                            break 'download;
                        }
                    }
                } else {
                    // 续传任务，先以追加模式打开
                    match OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&download_dir)
                    {
                        Ok(f) => {
                            // 校验文件大小：如果文件长度小于期望的偏移，说明文件异常，重置下载
                            if let Ok(meta) = f.metadata() {
                                if meta.len() < downloaded {
                                    // 文件被截断或损坏，清空文件并从头下载
                                    drop(f); // 先关闭文件，避免占用
                                    match OpenOptions::new()
                                        .write(true)
                                        .create(true)
                                        .truncate(true)
                                        .open(&download_dir)
                                    {
                                        Ok(new_f) => {
                                            downloaded = 0;
                                            Some(new_f)
                                        }
                                        Err(e) => {
                                            log::error!("文件重置失败: {}", e);
                                            progress::emit_error(
                                                &app_handle,
                                                &ctx.task_id,
                                                "文件异常，请重试",
                                            );
                                            break 'download;
                                        }
                                    }
                                } else {
                                    Some(f)
                                }
                            } else {
                                // 无法获取元数据，保守起见改为从头下载
                                drop(f);
                                match OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .truncate(true)
                                    .open(&download_dir)
                                {
                                    Ok(new_f) => {
                                        downloaded = 0;
                                        Some(new_f)
                                    }
                                    Err(e) => {
                                        log::error!("文件重置失败: {}", e);
                                        progress::emit_error(
                                            &app_handle,
                                            &ctx.task_id,
                                            "文件异常，请重试",
                                        );
                                        break 'download;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("文件打开失败: {}", e);
                            progress::emit_error(&app_handle, &ctx.task_id, "文件访问失败");
                            break 'download;
                        }
                    }
                }
            };

            if let Some(f) = f {
                // 包装为 BufWriter，减少磁盘 IO 次数
                file = Some(BufWriter::with_capacity(FILE_BUFFER_CAPACITY, f));

                // 更新 final_path：SAF 模式为 URI，普通模式为普通路径
                if is_saf {
                    if let Some(uri) = saf_file_uri.clone() {
                        *controller.final_path.lock().await = Some(uri);
                    }
                } else {
                    *controller.final_path.lock().await = Some(download_dir.clone());
                }
            } else {
                break 'download;
            }
        }

        // 发起下载请求（带网络重试）
        let mut attempt = 0;
        let response = loop {
            let mut request = DOWNLOAD_CLIENT
                .get(&url)
                .header("Referer", "https://y.qq.com");

            if downloaded > 0 {
                request = request.header(RANGE, format!("bytes={}-", downloaded));
            }
            match request.send().await {
                Ok(resp) => break resp,
                Err(e) => {
                    attempt += 1;
                    log::warn!(
                        "任务 {} 下载请求失败 (尝试 {}/3): {}",
                        ctx.task_id,
                        attempt,
                        e
                    );
                    if is_retryable_network_error(&e) && attempt < 3 {
                        tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
                        continue;
                    } else {
                        log::error!("任务 {} 最终下载请求失败: {}", ctx.task_id, e);
                        progress::emit_error(&app_handle, &ctx.task_id, "网络错误，请稍后重试");
                        break 'download;
                    }
                }
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
            // 刷新缓冲区
            if let Some(ref mut f) = file {
                if let Err(e) = f.flush() {
                    log::error!("刷新文件缓冲区失败: {}", e);
                    progress::emit_error(&app_handle, &ctx.task_id, "写入文件失败");
                    break 'download;
                }
            }
            let final_display_path = if is_saf {
                saf_file_uri.clone().unwrap_or_else(|| download_dir.clone())
            } else {
                download_dir.clone()
            };
            save_lyrics_sidecar(
                &app_handle,
                &ctx.song_id,
                is_saf,
                &download_dir,
                &saf_folder_uri,
            )
            .await;
            save_cover_art(
                &app_handle,
                is_saf,
                &download_dir,
                &saf_file_uri,
                &ctx.song_info.cover_url,
                &ctx.song_info.title,
                &ctx.song_info.artist,
                &ctx.song_info.album,
            )
            .await;
            let actual_quality =
                crate::utils::quality::quality_label_from_filename(&actual_quality_filename);
            let requested_quality =
                crate::utils::quality::quality_label_from_filename(&ctx.quality_filename);
            progress::emit_completed(
                &app_handle,
                &ctx.task_id,
                &final_display_path,
                saf_folder_uri.clone(),
                &actual_quality,
                &actual_quality_filename,
                Some(&requested_quality),
            );
            break 'download;
        }

        if status.is_client_error() || status.is_server_error() {
            if status == StatusCode::FORBIDDEN
                || status == StatusCode::GONE
                || status == StatusCode::NOT_FOUND
            {
                progress::emit_link_expired(&app_handle, &ctx.task_id, downloaded);
            } else {
                log::error!("任务 {} 服务器错误: {}", ctx.task_id, status);
                progress::emit_error(&app_handle, &ctx.task_id, "服务器错误，请稍后重试");
            }
            break 'download;
        }

        let mut stream = response.bytes_stream();
        let mut last_report = Instant::now();
        let mut last_downloaded = downloaded;
        let mut should_retry_stream = false;

        // 速度平滑：保存最近 5 次采样的速度值（B/s）
        let mut speed_samples: VecDeque<u64> = VecDeque::with_capacity(5);

        // 内部流读取循环
        loop {
            // 检查取消
            if controller.cancel_token.is_cancelled() {
                break 'download;
            }

            // 检查暂停：如果暂停，跳出内部循环，回到外层重新请求
            if controller.pause_flag.load(Ordering::SeqCst) {
                break; // 暂停跳出内部循环
            }

            let chunk_result = stream.next().await;
            let chunk = match chunk_result {
                Some(Ok(bytes)) => bytes,
                Some(Err(e)) => {
                    log::error!("任务 {} 读取流错误: {}", ctx.task_id, e);
                    // 如果还未超过流错误重试次数，标记为重试并跳出内部循环
                    if stream_retries < MAX_STREAM_RETRIES {
                        stream_retries += 1;
                        should_retry_stream = true;
                    } else {
                        progress::emit_error(&app_handle, &ctx.task_id, "网络错误，请稍后重试");
                    }
                    break;
                }
                None => break, // 流正常结束
            };

            // 转换为可变的 Vec<u8>
            let mut chunk_data = chunk.to_vec();
            let chunk_len = chunk_data.len() as u64;

            // 解密
            crypto::decrypt_chunk(&decrypt_ctx, &mut chunk_data, chunk_len, downloaded);

            // 写入文件
            if let Some(ref mut f) = file {
                if let Err(e) = f.write_all(&chunk_data) {
                    log::error!("写入文件错误: {}", e);
                    progress::emit_error(&app_handle, &ctx.task_id, "写入文件失败");
                    break 'download;
                }
            }

            downloaded += chunk_len;

            let now = Instant::now();
            let elapsed = now - last_report;
            if elapsed >= Duration::from_millis(500) {
                // 计算瞬时速度
                let instant_speed = if elapsed.as_secs_f64() > 0.0 {
                    ((downloaded - last_downloaded) as f64 / elapsed.as_secs_f64()) as u64
                } else {
                    0
                };

                // 加入采样队列并计算移动平均
                speed_samples.push_back(instant_speed);
                if speed_samples.len() > 5 {
                    speed_samples.pop_front();
                }
                let avg_speed = if speed_samples.is_empty() {
                    0
                } else {
                    speed_samples.iter().sum::<u64>() / speed_samples.len() as u64
                };

                progress::emit_progress(&app_handle, &ctx.task_id, downloaded, total, avg_speed);
                last_report = now;
                last_downloaded = downloaded;
            }

            if total > 0 && downloaded >= total {
                // 刷新缓冲区
                if let Some(ref mut f) = file {
                    if let Err(e) = f.flush() {
                        log::error!("刷新文件缓冲区失败: {}", e);
                        progress::emit_error(&app_handle, &ctx.task_id, "写入文件失败");
                        break 'download;
                    }
                }
                log::info!("下载完成: {}", download_dir);
                let final_display_path = if is_saf {
                    saf_file_uri.clone().unwrap_or_else(|| download_dir.clone())
                } else {
                    download_dir.clone()
                };
                save_lyrics_sidecar(
                    &app_handle,
                    &ctx.song_id,
                    is_saf,
                    &download_dir,
                    &saf_folder_uri,
                )
                .await;
                save_cover_art(
                    &app_handle,
                    is_saf,
                    &download_dir,
                    &saf_file_uri,
                    &ctx.song_info.cover_url,
                    &ctx.song_info.title,
                    &ctx.song_info.artist,
                    &ctx.song_info.album,
                )
                .await;
                let actual_quality =
                    crate::utils::quality::quality_label_from_filename(&actual_quality_filename);
                let requested_quality =
                    crate::utils::quality::quality_label_from_filename(&ctx.quality_filename);
                progress::emit_completed(
                    &app_handle,
                    &ctx.task_id,
                    &final_display_path,
                    saf_folder_uri.clone(),
                    &actual_quality,
                    &actual_quality_filename,
                    Some(&requested_quality),
                );
                break 'download;
            }
        }

        // 内部循环结束后的处理
        if controller.pause_flag.load(Ordering::SeqCst) {
            // 因暂停跳出，等待恢复
            if wait_for_resume_async(&controller).await {
                break 'download; // 被取消
            }
            // 恢复后需要重新获取链接，清空 url 与 key，并释放当前文件句柄
            url.clear();
            key.clear();
            file = None; // 释放 BufWriter 并自动 flush
            continue 'download;
        }

        // 如果是因为流错误触发的重试
        if should_retry_stream {
            // 重新获取链接，避免旧链接过期
            url.clear();
            key.clear();
            file = None; // 释放 BufWriter 并自动 flush
            continue 'download;
        }

        // 其他情况，直接退出
        break 'download;
    }

    // 显式关闭文件句柄，释放资源
    drop(file);

    // 如果任务被取消且用户要求删除文件，执行删除
    if controller.cancel_token.is_cancelled()
        && controller.delete_file_on_cancel.load(Ordering::SeqCst)
    {
        if is_saf {
            // SAF 模式：使用插件 API 删除
            if let Some(uri) = saf_file_uri.clone() {
                let fs_uri = FsUri::from_uri(uri);
                let api = app_handle.android_fs();
                if let Err(e) = api.remove_file(&fs_uri) {
                    log::error!("删除 SAF 文件失败: {}", e);
                } else {
                    log::info!("SAF 文件已删除: {}", fs_uri.uri);
                }
            } else {
                log::warn!("任务 {} 取消时未记录 SAF 文件 URI，无法删除", ctx.task_id);
            }
            // 同步删除同名 .lrc
            if let Some(folder) = &saf_folder_uri {
                delete_saf_lrc(&app_handle, folder, &download_dir);
            }
        } else {
            // 普通模式：使用标准库删除
            if let Err(e) = fs::remove_file(&download_dir) {
                log::error!("删除文件失败: {}", e);
            } else {
                log::info!("文件已成功删除: {}", download_dir);
            }
            delete_sidecar_lrc(&download_dir);
        }
    }
}

// ==================== 辅助函数 ====================

/// 获取下载目录（绝对路径）及文件命名模板
pub(crate) async fn get_download_settings(
    app_handle: &AppHandle,
) -> (String, String, Option<String>) {
    use crate::storage::store_wrapper;

    let default_dir = crate::commands::file_ops::get_default_download_dir_impl(app_handle);
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

    // 过滤无效路径（Android 应用私有目录）
    let dir = if dir.contains("/data/user/0/") || dir.contains("/data/data/") {
        log::warn!("检测到应用私有目录路径，已回退为默认下载目录: {}", dir);
        default_dir
    } else if Path::new(&dir).is_absolute() || dir == "saf://" {
        dir
    } else {
        log::warn!(
            "下载目录不是绝对路径，已回退为默认下载目录: {}",
            default_dir
        );
        default_dir
    };

    let saf_folder_uri = settings
        .get("safFolderUri")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let template = settings
        .get("namingTemplate")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_template)
        .to_string();

    (dir, template, saf_folder_uri)
}

/// 将加密文件扩展名映射为解密后的真实扩展名
pub(crate) fn map_decrypted_extension(ext: &str) -> &str {
    match ext {
        "mgg" => "ogg",
        "mflac" => "flac",
        // 未知则保持原样
        _ => ext,
    }
}

/// 删除音频同名的 `.lrc`，文件不存在时忽略。
pub(crate) fn delete_sidecar_lrc(audio_path: &str) {
    let lrc = crate::utils::quality::sidecar_lrc_path(audio_path);
    match fs::remove_file(&lrc) {
        Ok(()) => log::info!("已删除歌词: {}", lrc.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => log::warn!("删除歌词失败 {}: {}", lrc.display(), e),
    }
}

/// SAF 模式下删除音频同名的 `.lrc`。
/// folder_json 是父目录的 FsUri JSON，audio_name 是音频文件名（含扩展名）。
pub(crate) fn delete_saf_lrc(app_handle: &AppHandle, folder_json: &str, audio_name: &str) {
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app_handle, folder_json, audio_name);
    }

    #[cfg(target_os = "android")]
    {
        let parent_uri = match FsUri::from_json_str(folder_json) {
            Ok(uri) => uri,
            Err(e) => {
                log::warn!("解析 SAF 目录失败，无法删除歌词: {}", e);
                return;
            }
        };
        let lrc_name = crate::utils::quality::sidecar_lrc_path(audio_name);
        let api = app_handle.android_fs();
        // 尝试在父目录中查找已存在的 .lrc
        if let Ok(lrc_uri) = api.resolve_file_uri(&parent_uri, &lrc_name) {
            if let Err(e) = api.remove_file(&lrc_uri) {
                log::warn!("删除 SAF 歌词失败: {}", e);
            } else {
                log::info!("已删除 SAF 歌词: {}", lrc_name.display());
            }
        } else {
            // 歌词不存在，无需删除
            log::debug!("SAF 歌词不存在，无需删除: {}", lrc_name.display());
        }
    }
}

/// 下载完成后写入同名 LRC。失败只记日志，不影响音频任务。
fn lyrics_enabled(app_handle: &AppHandle) -> bool {
    use crate::storage::store_wrapper;
    store_wrapper::load_string(app_handle, "settings")
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
        .and_then(|v| v.get("downloadLyrics")?.as_bool())
        .unwrap_or(true)
}

async fn save_lyrics_sidecar(
    app_handle: &AppHandle,
    song_id: &str,
    is_saf: bool,
    audio_path: &str,
    saf_folder_uri: &Option<String>,
) {
    if !lyrics_enabled(app_handle) {
        return;
    }
    match crate::commands::api::fetch_lyrics_text(song_id).await {
        Ok(lyric) => {
            if is_saf {
                if let Err(e) = write_saf_lrc(app_handle, audio_path, saf_folder_uri, &lyric) {
                    log::warn!("写入 SAF 歌词失败: {}", e);
                }
            } else {
                let lrc_path = crate::utils::quality::sidecar_lrc_path(audio_path);
                if let Err(e) = fs::write(&lrc_path, lyric.as_bytes()) {
                    log::warn!("写入歌词失败 {}: {}", lrc_path.display(), e);
                } else {
                    log::info!("已写入歌词: {}", lrc_path.display());
                }
            }
        }
        Err(e) => log::warn!("获取歌词失败 ({}): {}", song_id, e),
    }
}

fn write_saf_lrc(
    app_handle: &AppHandle,
    audio_file_name: &str,
    saf_folder_uri: &Option<String>,
    lyric: &str,
) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    {
        let _ = (app_handle, audio_file_name, saf_folder_uri, lyric);
        Ok(())
    }

    #[cfg(target_os = "android")]
    {
        let folder = saf_folder_uri.as_ref().ok_or("缺少 SAF 目录")?;
        let parent_uri = FsUri::from_json_str(folder).map_err(|e| e.to_string())?;
        let lrc_name = crate::utils::quality::sidecar_lrc_path(audio_file_name);
        let api = app_handle.android_fs();
        let file_uri = match api.resolve_file_uri(&parent_uri, &lrc_name) {
            Ok(uri) => uri,
            Err(_) => api
                .create_new_file(&parent_uri, &lrc_name, Some("application/octet-stream"))
                .map_err(|e| e.to_string())?,
        };
        let mut f = api
            .open_file_writable(&file_uri)
            .map_err(|e| e.to_string())?;
        use std::io::Write;
        f.write_all(lyric.as_bytes())
            .map_err(|e| format!("写入 SAF 歌词失败: {}", e))?;
        Ok(())
    }
}

/// 下载封面图字节并嵌入音频标签。
/// 普通模式与 SAF（Android）模式均支持：SAF 模式通过 SAF 文件 URI 打开为可读写
/// 文件句柄，由 lofty 原地修改标签。
async fn save_cover_art(
    app_handle: &AppHandle,
    is_saf: bool,
    audio_path: &str,
    saf_file_uri: &Option<String>,
    cover_url: &str,
    title: &str,
    artist: &str,
    album: &str,
) {
    if cover_url.is_empty() {
        log::debug!("封面 URL 为空，跳过封面嵌入: {}", audio_path);
        return;
    }

    let bytes = match fetch_cover_bytes(cover_url).await {
        Ok(b) => b,
        Err(e) => {
            log::warn!("获取封面失败 {}: {}", cover_url, e);
            return;
        }
    };
    if bytes.is_empty() {
        log::warn!("封面数据为空，跳过嵌入: {}", cover_url);
        return;
    }

    let result = if is_saf {
        embed_cover_art_saf(app_handle, saf_file_uri, &bytes, title, artist, album)
    } else {
        embed_cover_art_path(audio_path, &bytes, title, artist, album)
    };

    match result {
        Ok(()) => log::info!("已嵌入封面与标签: {}", audio_path),
        Err(e) => log::warn!("嵌入封面失败 {}: {}", audio_path, e),
    }
}

/// 下载封面图片字节
async fn fetch_cover_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = DOWNLOAD_CLIENT
        .get(url)
        .header("Referer", "https://y.qq.com")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("封面响应状态: {}", resp.status()));
    }
    let body = resp
        .bytes()
        .await
        .map_err(|e| format!("读取封面失败: {}", e))?;
    Ok(body.to_vec())
}

/// 普通模式：用文件路径读写标签
fn embed_cover_art_path(
    audio_path: &str,
    cover_bytes: &[u8],
    title: &str,
    artist: &str,
    album: &str,
) -> Result<(), String> {
    use lofty::config::WriteOptions;
    use lofty::file::{AudioFile, TaggedFileExt};

    let path = std::path::Path::new(audio_path);
    let mut tagged_file = lofty::read_from_path(path)
        .map_err(|e| format!("读取音频失败: {}", e))?;

    ensure_primary_tag(&mut tagged_file);

    let tag = tagged_file
        .primary_tag_mut()
        .ok_or_else(|| "无主标签可写".to_string())?;
    set_tag_fields(tag, cover_bytes, title, artist, album);

    tagged_file
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("保存标签失败: {}", e))?;
    Ok(())
}

/// SAF（Android）模式：读取整个 SAF 文件字节，在内存中用 lofty 修改标签，
/// 然后全量写回。避免 SAF 句柄 truncate 语义不确定的风险。
#[cfg(target_os = "android")]
fn embed_cover_art_saf(
    app_handle: &AppHandle,
    saf_file_uri: &Option<String>,
    cover_bytes: &[u8],
    title: &str,
    artist: &str,
    album: &str,
) -> Result<(), String> {
    use std::io::Cursor;

    let uri_str = saf_file_uri
        .as_ref()
        .ok_or_else(|| "缺少 SAF 文件 URI".to_string())?;
    let api = app_handle.android_fs();
    let fs_uri = FsUri::from_uri(uri_str.clone());

    // 读取整个文件到内存
    let data = api
        .read(&fs_uri)
        .map_err(|e| format!("读取 SAF 文件失败: {}", e))?;
    let mut cursor = Cursor::new(data);

    let new_data = embed_cover_in_cursor(&mut cursor, cover_bytes, title, artist, album)?;
    api.write(&fs_uri, &new_data)
        .map_err(|e| format!("写回 SAF 文件失败: {}", e))?;
    Ok(())
}

/// 在内存游标上完成封面嵌入，返回修改后的字节。桌面/Android 共用，便于桌面编译验证。
#[allow(dead_code)]
fn embed_cover_in_cursor(
    cursor: &mut std::io::Cursor<Vec<u8>>,
    cover_bytes: &[u8],
    title: &str,
    artist: &str,
    album: &str,
) -> Result<Vec<u8>, String> {
    use lofty::config::WriteOptions;
    use lofty::file::{AudioFile, TaggedFileExt};
    use std::io::{Seek, SeekFrom};

    let mut tagged_file = lofty::probe::Probe::new(&mut *cursor)
        .guess_file_type()
        .map_err(|e| format!("探测音频类型失败: {}", e))?
        .read()
        .map_err(|e| format!("读取音频失败: {}", e))?;
    ensure_primary_tag(&mut tagged_file);

    let tag = tagged_file
        .primary_tag_mut()
        .ok_or_else(|| "无主标签可写".to_string())?;
    set_tag_fields(tag, cover_bytes, title, artist, album);

    cursor
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("文件定位失败: {}", e))?;
    tagged_file
        .save_to(cursor, WriteOptions::default())
        .map_err(|e| format!("保存标签失败: {}", e))?;

    Ok(std::mem::take(cursor.get_mut()))
}

/// SAF 模式在非 Android 平台的占位实现（编译占位，运行时不会走到）
#[cfg(not(target_os = "android"))]
fn embed_cover_art_saf(
    _app_handle: &AppHandle,
    _saf_file_uri: &Option<String>,
    _cover_bytes: &[u8],
    _title: &str,
    _artist: &str,
    _album: &str,
) -> Result<(), String> {
    Err("SAF 模式仅在 Android 上可用".to_string())
}

/// 若不存在主标签，按文件类型创建并插入一个空标签
fn ensure_primary_tag(tagged_file: &mut lofty::file::TaggedFile) {
    use lofty::file::TaggedFileExt;
    use lofty::tag::{Tag, TagType};
    if tagged_file.primary_tag().is_some() {
        return;
    }
    let tag_type = tagged_file.primary_tag_type();
    if tag_type == TagType::Id3v2 {
        tagged_file.insert_tag(Tag::new(TagType::Id3v2));
    } else {
        tagged_file.insert_tag(Tag::new(tag_type));
    }
}

/// 写入文本字段与封面
fn set_tag_fields(
    tag: &mut lofty::tag::Tag,
    cover_bytes: &[u8],
    title: &str,
    artist: &str,
    album: &str,
) {
    use lofty::picture::{MimeType, Picture, PictureType};
    use lofty::tag::Accessor;

    if !title.is_empty() {
        tag.set_title(title.to_string());
    }
    if !artist.is_empty() {
        tag.set_artist(artist.to_string());
    }
    if !album.is_empty() {
        tag.set_album(album.to_string());
    }
    tag.remove_picture_type(PictureType::CoverFront);
    let pic = Picture::new_unchecked(
        PictureType::CoverFront,
        Some(MimeType::Jpeg),
        None,
        cover_bytes.to_vec(),
    );
    tag.push_picture(pic);
}
