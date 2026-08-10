// 所有品质标签，按从低到高排序
export const ALL_QUALITY_ORDER: string[] = [
    '48kacc',
    '96kacc',
    '192kacc',
    '96kogg',
    '192kogg',
    '128kmp3',
    '320kmp3',
    'ape',
    'flac',
    'hires',
    '杜比全景声',
    '臻品全景声',
    '臻品母带',
]

/** 降级顺序：从高到低 */
export const QUALITY_DOWNGRADE_ORDER: string[] = [...ALL_QUALITY_ORDER].reverse()

export type Quality = string  // 不再限制字面量，兼容所有后端标签

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
    quality: string   // 品质标签，如 "128kmp3", "flac", "臻品母带" 等
    filename: string  // 对应下载文件名，如 "M800xxxx.mp3"
    size: number      // 文件字节大小
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
    filename: string           // 实际下载的品质文件名
    quality: Quality           // 实际选择的品质标签
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

export const DEFAULT_SETTINGS: Settings = {
    defaultQuality: 'ask',
    autoDowngrade: true,
    downloadDir: '',
    namingTemplate: '{song} - {artist}',
    maxConcurrent: 3,
    jumpToTask: true,
}