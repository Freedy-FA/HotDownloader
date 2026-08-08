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

export interface SongInfo {
    id: string
    title: string
    artist: string
    album: string
    coverUrl: string
}

export interface TaskRecord {
    id: string
    songId: string
    songTitle: string
    artist: string
    album: string
    coverUrl: string
    quality: Quality
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