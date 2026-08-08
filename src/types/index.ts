export type Quality = 'Hi-Res' | 'FLAC' | '320k' | '192k' | '128k' | 'ask'

export type TaskStatus = 'waiting' | 'downloading' | 'paused' | 'completed' | 'error'

export interface Settings {
    defaultQuality: Quality
    autoDowngrade: boolean
    downloadDir: string
    namingTemplate: string
    maxConcurrent: number
    jumpToTask: boolean
}

/** 歌曲可用的单个品质项 */
export interface QualityItem {
    quality: string   // 品质标签，如 "128k", "FLAC", "Hi-Res", "臻品母带" 等
    filename: string  // 对应下载文件名，如 "M800xxxx.mp3"
}

export interface SongInfo {
    id: string
    title: string
    artist: string
    album: string
    coverUrl: string
    mediaMid: string
    qualities: QualityItem[]
}

export interface TaskRecord {
    id: string
    songId: string
    songTitle: string
    artist: string
    album: string
    coverUrl: string
    mediaMid: string           // 用于后续可能的操作
    filename: string           // 实际下载的文件名，重试时直接使用
    quality: Quality           // 目标音质（用户期望的品质，实际可能因降级而不同）
    status: TaskStatus
    errorMsg?: string
    filePath?: string
    fileSize: number
    downloaded: number
    retryCount: number
    addedAt: number
}

export interface DownloadProgressPayload {
    task_id: string
    downloaded: number
    total: number
    speed: number
}

export interface DownloadCompletedPayload {
    task_id: string
    final_path: string
}

export interface DownloadErrorPayload {
    task_id: string
    error_msg: string
}

export interface DownloadLinkExpiredPayload {
    task_id: string
    current_offset: number
}

export const QUALITY_DOWNGRADE_ORDER: Quality[] = [
    'Hi-Res',
    'FLAC',
    '320k',
    '192k',
    '128k',
]

export const DEFAULT_SETTINGS: Settings = {
    defaultQuality: '320k',
    autoDowngrade: true,
    downloadDir: '',
    namingTemplate: '{song} - {artist}',
    maxConcurrent: 3,
    jumpToTask: true,
}