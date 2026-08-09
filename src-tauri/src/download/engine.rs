use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

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
}

#[derive(Clone)]
pub struct DownloadEngine {
    pub app_handle: AppHandle,
    pending_tasks: Arc<Mutex<VecDeque<TaskContext>>>,
    ready_tasks: Arc<Mutex<VecDeque<TaskContext>>>,
    active_controllers: Arc<Mutex<HashMap<String, TaskController>>>,
    max_concurrent: Arc<AtomicU32>,
    active_downloads: Arc<AtomicU32>,
    scheduler_notify: Arc<Notify>,
}

impl DownloadEngine {
    pub fn new(app_handle: AppHandle) -> Self {
        DownloadEngine {
            app_handle,
            pending_tasks: Arc::new(Mutex::new(VecDeque::new())),
            ready_tasks: Arc::new(Mutex::new(VecDeque::new())),
            active_controllers: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent: Arc::new(AtomicU32::new(3)),
            active_downloads: Arc::new(AtomicU32::new(0)),
            scheduler_notify: Arc::new(Notify::new()),
        }
    }

    /// 异步添加任务，同时预计算最终保存路径
    pub async fn add_task(
        &self,
        task_id: String,
        url: String,
        save_path: String,
        quality: String,
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
        };

        let ctx = TaskContext {
            task_id: task_id.clone(),
            url,
            save_path,
            quality,
            key,
            file_size,
            downloaded_offset: 0,
            app_handle: self.app_handle.clone(),
            song_info: super::task::SongInfo {
                title: song_title,
                artist,
                album,
            },
        };

        self.active_controllers
            .lock()
            .await
            .insert(task_id, controller);
        self.pending_tasks.lock().await.push_back(ctx);
        self.scheduler_notify.notify_one();
    }

    /// 异步更新任务 URL 并移入就绪队列
    pub async fn update_task(&self, task_id: &str, url: String, key: String, offset: u64) {
        let mut pending = self.pending_tasks.lock().await;
        if let Some(pos) = pending.iter().position(|t| t.task_id == task_id) {
            let mut ctx = pending.remove(pos).unwrap();
            ctx.url = url;
            ctx.key = key;
            ctx.downloaded_offset = offset;
            drop(pending); // 释放锁，避免死锁
            self.ready_tasks.lock().await.push_back(ctx);
        }

        // 通知该任务的 url_ready（如果有）
        if let Some(ctrl) = self.active_controllers.lock().await.get(task_id) {
            ctrl.url_ready.notify_one();
        }

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

    /// 取消任务：设置取消标志，同时标记是否需要删除文件
    pub async fn cancel(&self, task_id: &str, delete_file: bool) {
        log::info!("取消任务 {} (delete_file={})", task_id, delete_file);
        if let Some(ctrl) = self.active_controllers.lock().await.get(task_id) {
            ctrl.cancel_token.cancel();
            // 将删除意图传递给下载线程
            ctrl.delete_file_on_cancel.store(delete_file, Ordering::SeqCst);
            // 如果任务处于暂停等待状态，需要唤醒它以便退出循环
            ctrl.resume_notify.notify_one();
            ctrl.url_ready.notify_one();
        }

        // 清理队列
        self.pending_tasks
            .lock()
            .await
            .retain(|t| t.task_id != task_id);
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
                    })
                };

                if let Some(ctrl) = ctrl {
                    self.active_downloads.fetch_add(1, Ordering::SeqCst);
                    let active_downloads = self.active_downloads.clone();
                    let scheduler_notify = self.scheduler_notify.clone();
                    let app_handle = self.app_handle.clone();

                    tokio::spawn(async move {
                        download_task(ctx, ctrl, app_handle).await;
                        active_downloads.fetch_sub(1, Ordering::SeqCst);
                        scheduler_notify.notify_one();
                    });
                }
                // 如果 ctrl 不存在，跳过该任务（可能已被取消）
            }

            self.scheduler_notify.notified().await;
        }
    }
}