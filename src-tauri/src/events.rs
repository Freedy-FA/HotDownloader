use serde::Serialize;

pub const DOWNLOAD_PROGRESS: &str = "download-progress";
pub const DOWNLOAD_COMPLETED: &str = "download-completed";
pub const DOWNLOAD_ERROR: &str = "download-error";
pub const DOWNLOAD_LINK_EXPIRED: &str = "download-link-expired";

#[derive(Serialize, Clone)]
pub struct DownloadProgressPayload {
    pub task_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
}

#[derive(Serialize, Clone)]
pub struct DownloadCompletedPayload {
    pub task_id: String,
    pub final_path: String,
}

#[derive(Serialize, Clone)]
pub struct DownloadErrorPayload {
    pub task_id: String,
    pub error_msg: String,
}

#[derive(Serialize, Clone)]
pub struct DownloadLinkExpiredPayload {
    pub task_id: String,
    pub current_offset: u64,
}