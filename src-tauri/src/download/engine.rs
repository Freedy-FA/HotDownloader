use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use tauri::AppHandle;
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;

use super::task::{download_task, TaskContext};

pub struct TaskController {
    pub cancel_token: CancellationToken,
    pub pause_flag: Arc<AtomicBool>,
    pub resume_notify: Arc<Notify>,
    pub url_ready: Arc<Notify>,
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

    pub fn add_task(
        &self,
        task_id: String,
        url: String,
        save_path: String,
        quality: String,
        key: String,
        file_size: u64,
    ) {
        let controller = TaskController {
            cancel_token: CancellationToken::new(),
            pause_flag: Arc::new(AtomicBool::new(false)),
            resume_notify: Arc::new(Notify::new()),
            url_ready: Arc::new(Notify::new()),
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
            // song_info 需要从某处传入，目前使用默认值；实际应该由前端传入
            song_info: super::task::SongInfo {
                title: String::new(),
                artist: String::new(),
                album: String::new(),
            },
        };

        // 如果 url 非空，说明可以直接放入 ready_tasks，但命令约定添加时 url 为空
        // 这里统一放入 pending_tasks
        let pending = self.pending_tasks.clone();
        let controllers = self.active_controllers.clone();
        let notify = self.scheduler_notify.clone();

        tokio::spawn(async move {
            controllers.lock().await.insert(task_id, controller);
            pending.lock().await.push_back(ctx);
            notify.notify_one();
        });
    }

    pub fn update_task(&self, task_id: &str, url: String, key: String, offset: u64) {
        let pending = self.pending_tasks.clone();
        let ready = self.ready_tasks.clone();
        let controllers = self.active_controllers.clone();
        let notify = self.scheduler_notify.clone();
        let tid = task_id.to_string();

        tokio::spawn(async move {
            // 从 pending_tasks 中取出并更新
            let mut pending_queue = pending.lock().await;
            if let Some(pos) = pending_queue.iter().position(|t| t.task_id == tid) {
                let mut ctx = pending_queue.remove(pos).unwrap();
                ctx.url = url;
                ctx.key = key;
                ctx.downloaded_offset = offset;
                // 放入 ready_tasks
                ready.lock().await.push_back(ctx);
            }
            // 通知该任务的 url_ready
            if let Some(ctrl) = controllers.lock().await.get(&tid) {
                ctrl.url_ready.notify_one();
            }
            // 唤醒调度器
            notify.notify_one();
        });
    }

    pub fn pause(&self, task_id: &str) {
        let controllers = self.active_controllers.clone();
        let tid = task_id.to_string();
        tokio::spawn(async move {
            if let Some(ctrl) = controllers.lock().await.get(&tid) {
                ctrl.pause_flag.store(true, Ordering::SeqCst);
            }
        });
    }

    pub fn resume(&self, task_id: &str) {
        let controllers = self.active_controllers.clone();
        let tid = task_id.to_string();
        tokio::spawn(async move {
            if let Some(ctrl) = controllers.lock().await.get(&tid) {
                ctrl.pause_flag.store(false, Ordering::SeqCst);
                ctrl.resume_notify.notify_one();
            }
        });
    }

    pub fn cancel(&self, task_id: &str) {
        let controllers = self.active_controllers.clone();
        let pending = self.pending_tasks.clone();
        let ready = self.ready_tasks.clone();
        let tid = task_id.to_string();

        tokio::spawn(async move {
            // 触发取消令牌
            if let Some(ctrl) = controllers.lock().await.remove(&tid) {
                ctrl.cancel_token.cancel();
            }
            // 从队列中移除
            pending.lock().await.retain(|t| t.task_id != tid);
            ready.lock().await.retain(|t| t.task_id != tid);
        });
    }

    pub fn set_concurrency(&self, max: u32) {
        self.max_concurrent.store(max, Ordering::SeqCst);
        // 可能现在可以启动新任务
        self.scheduler_notify.notify_one();
    }

    pub async fn run_scheduler(&self) {
        loop {
            // 尝试启动尽可能多的就绪任务
            while let Some(ctx) = {
                let mut ready = self.ready_tasks.lock().await;
                ready.pop_front()
            } {
                let current = self.active_downloads.load(Ordering::SeqCst);
                let max = self.max_concurrent.load(Ordering::SeqCst);
                if current >= max {
                    // 放回队列头部，等待下次通知
                    self.ready_tasks.lock().await.push_front(ctx);
                    break;
                }

                // 获取对应的控制器（必须存在，否则跳过）
                let ctrl = {
                    let controllers = self.active_controllers.lock().await;
                    controllers.get(&ctx.task_id).map(|c| TaskController {
                        cancel_token: c.cancel_token.clone(),
                        pause_flag: c.pause_flag.clone(),
                        resume_notify: c.resume_notify.clone(),
                        url_ready: c.url_ready.clone(),
                    })
                };

                match ctrl {
                    Some(ctrl) => {
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
                    None => {
                        // 控制器已被移除，忽略该任务
                        continue;
                    }
                }
            }

            // 等待新任务加入或某个下载完成
            self.scheduler_notify.notified().await;
        }
    }
}