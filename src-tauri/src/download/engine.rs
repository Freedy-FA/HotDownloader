use std::collections::{HashMap, VecDeque};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use log;
use tauri::AppHandle;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

use super::task::{download_task, TaskContext};

pub struct TaskController {
    pub cancel_token: CancellationToken,
    pub pause_flag: Arc<AtomicBool>,
    pub resume_notify: Arc<Notify>,
    pub url_ready: Arc<Notify>,
    /// 取消时是否需要删除文件
    pub delete_file_on_cancel: Arc<AtomicBool>,
    /// 下载线程确定的最终文件路径（供外部删除使用）
    pub final_path: Arc<Mutex<Option<String>>>,
}

#[derive(Clone)]
pub struct DownloadEngine {
    pub app_handle: AppHandle,
    ready_tasks: Arc<Mutex<VecDeque<TaskContext>>>,
    active_controllers: Arc<Mutex<HashMap<String, TaskController>>>,
    max_concurrent: Arc<AtomicU32>,
    active_downloads: Arc<AtomicU32>,
    scheduler_notify: Arc<Notify>,
    task_contexts: Arc<Mutex<HashMap<String, TaskContext>>>,
}

impl DownloadEngine {
    pub fn new(app_handle: AppHandle) -> Self {
        DownloadEngine {
            app_handle,
            ready_tasks: Arc::new(Mutex::new(VecDeque::new())),
            active_controllers: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent: Arc::new(AtomicU32::new(3)),
            active_downloads: Arc::new(AtomicU32::new(0)),
            scheduler_notify: Arc::new(Notify::new()),
            task_contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 异步添加任务，同时预计算最终保存路径
    pub async fn add_task(
        &self,
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
    ) {
        let controller = TaskController {
            cancel_token: CancellationToken::new(),
            pause_flag: Arc::new(AtomicBool::new(false)),
            resume_notify: Arc::new(Notify::new()),
            url_ready: Arc::new(Notify::new()),
            delete_file_on_cancel: Arc::new(AtomicBool::new(false)),
            final_path: Arc::new(Mutex::new(None)),
        };

        let ctx = TaskContext {
            task_id: task_id.clone(),
            song_id,
            url,
            save_path,
            quality: quality.clone(), // 保留原 quality 字段
            quality_filename: filename,
            key,
            file_size,
            downloaded_offset: 0,
            app_handle: self.app_handle.clone(),
            song_info: super::task::SongInfo {
                title: song_title,
                artist,
                album,
                quality, // 传入品质
            },
            final_path: controller.final_path.clone(), // 共享路径
        };

        self.task_contexts
            .lock()
            .await
            .insert(task_id.clone(), ctx.clone());
        self.active_controllers
            .lock()
            .await
            .insert(task_id, controller);
        self.ready_tasks.lock().await.push_back(ctx);
        self.scheduler_notify.notify_one();
    }

    /// 异步更新任务 URL 并移入就绪队列（重试时调用）
    pub async fn enqueue_task(&self, task_id: &str, offset: u64) {
        // 获取任务上下文（克隆后修改）
        let mut ctx = match self.task_contexts.lock().await.get(task_id).cloned() {
            Some(c) => c,
            None => return,
        };
        ctx.downloaded_offset = offset;
        ctx.url.clear(); // 强制在下载线程中重新获取链接

        // 如果任务之前已经结束，控制器可能已被移除；
        // 为了能让调度器再次启动该任务，需要重新创建一个控制器。
        // 保留与上下文共享的 final_path，确保后续删除文件能找到路径。
        let controller = TaskController {
            cancel_token: CancellationToken::new(),
            pause_flag: Arc::new(AtomicBool::new(false)),
            resume_notify: Arc::new(Notify::new()),
            url_ready: Arc::new(Notify::new()),
            delete_file_on_cancel: Arc::new(AtomicBool::new(false)),
            final_path: ctx.final_path.clone(), // 共享同一个 final_path
        };

        // 重新插入控制器（覆盖可能存在的旧控制器）
        self.active_controllers
            .lock()
            .await
            .insert(task_id.to_string(), controller);
        // 更新上下文（保存偏移量等修改）
        self.task_contexts
            .lock()
            .await
            .insert(task_id.to_string(), ctx.clone());
        // 放入就绪队列
        self.ready_tasks.lock().await.push_back(ctx);
        self.scheduler_notify.notify_one();
    }

    /// 异步暂停任务
    pub async fn pause(&self, task_id: &str) {
        if let Some(ctrl) = self.active_controllers.lock().await.get(task_id) {
            ctrl.pause_flag.store(true, Ordering::SeqCst);
        }
    }

    /// 异步恢复任务
    pub async fn resume(&self, task_id: &str) {
        if let Some(ctrl) = self.active_controllers.lock().await.get(task_id) {
            ctrl.pause_flag.store(false, Ordering::SeqCst);
            ctrl.resume_notify.notify_one();
        }
    }

    /// 取消任务（下载线程自行处理文件删除）
    pub async fn cancel(&self, task_id: &str, delete_file: bool) {
        log::info!("取消任务 {} (delete_file={})", task_id, delete_file);
        if let Some(ctrl) = self.active_controllers.lock().await.get(task_id) {
            ctrl.cancel_token.cancel();
            // 将删除意图传递给下载线程
            ctrl.delete_file_on_cancel
                .store(delete_file, Ordering::SeqCst);
            // 如果任务处于暂停等待状态，需要唤醒它以便退出循环
            ctrl.resume_notify.notify_one();
            ctrl.url_ready.notify_one();
        }

        // 清理队列
        self.ready_tasks
            .lock()
            .await
            .retain(|t| t.task_id != task_id);

        // 注意：不再在此处删除文件，改为 download_task 完成后自行处理
    }

    /// 设置并发数（同步，无需 Tokio 上下文）
    pub fn set_concurrency(&self, max: u32) {
        self.max_concurrent.store(max, Ordering::SeqCst);
        self.scheduler_notify.notify_one();
    }

    /// 调度循环（在后台 Tokio 任务中运行）
    pub async fn run_scheduler(&self) {
        loop {
            // 启动尽可能多的就绪任务
            while let Some(ctx) = {
                let mut ready = self.ready_tasks.lock().await;
                ready.pop_front()
            } {
                let current = self.active_downloads.load(Ordering::SeqCst);
                let max = self.max_concurrent.load(Ordering::SeqCst);
                if current >= max {
                    self.ready_tasks.lock().await.push_front(ctx);
                    break;
                }

                let ctrl = {
                    let controllers = self.active_controllers.lock().await;
                    controllers.get(&ctx.task_id).map(|c| TaskController {
                        cancel_token: c.cancel_token.clone(),
                        pause_flag: c.pause_flag.clone(),
                        resume_notify: c.resume_notify.clone(),
                        url_ready: c.url_ready.clone(),
                        delete_file_on_cancel: c.delete_file_on_cancel.clone(),
                        final_path: c.final_path.clone(),
                    })
                };

                if let Some(ctrl) = ctrl {
                    self.active_downloads.fetch_add(1, Ordering::SeqCst);
                    let active_downloads = self.active_downloads.clone();
                    let scheduler_notify = self.scheduler_notify.clone();
                    let app_handle = self.app_handle.clone();
                    let task_id = ctx.task_id.clone();
                    let engine = self.clone();

                    tokio::spawn(async move {
                        download_task(ctx, ctrl, app_handle).await;
                        // 下载结束（完成/错误），仅移除控制器，保留任务上下文供删除文件使用
                        engine.active_controllers.lock().await.remove(&task_id);
                        active_downloads.fetch_sub(1, Ordering::SeqCst);
                        scheduler_notify.notify_one();
                    });
                }
                // 如果 ctrl 不存在，跳过该任务（可能已被取消）
            }

            self.scheduler_notify.notified().await;
        }
    }

    /// 移除任务记录，并可选删除文件
    pub async fn remove(&self, task_id: &str, delete_file: bool) {
        // 先清理就绪队列
        self.ready_tasks
            .lock()
            .await
            .retain(|t| t.task_id != task_id);

        // 尝试从活跃控制器获取 final_path（任务可能仍在运行）
        let ctrl = self.active_controllers.lock().await.remove(task_id);
        if let Some(ctrl) = ctrl {
            if delete_file {
                let path_guard = ctrl.final_path.lock().await;
                if let Some(p) = path_guard.as_ref() {
                    if let Err(e) = fs::remove_file(p) {
                        log::error!("删除文件失败 {}: {}", p, e);
                    } else {
                        log::info!("已删除文件: {}", p);
                    }
                }
            }
        } else {
            // 任务不在活跃列表（已结束），从 task_contexts 中获取 final_path
            if delete_file {
                let ctx_map = self.task_contexts.lock().await;
                if let Some(ctx) = ctx_map.get(task_id) {
                    let path_guard = ctx.final_path.lock().await;
                    if let Some(p) = path_guard.as_ref() {
                        if let Err(e) = fs::remove_file(p) {
                            log::error!("删除文件失败 {}: {}", p, e);
                        } else {
                            log::info!("已删除文件: {}", p);
                        }
                    }
                }
            }
        }

        // 无论如何，清除任务上下文
        self.task_contexts.lock().await.remove(task_id);
    }
}
