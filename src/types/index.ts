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
    // 新增 SAF 文件夹 URI 和名称
    safFolderUri?: string
    safFolderName?: string
    writeMetadata: boolean
}

/** 歌曲可用的单个品质项 */
export interface QualityItem {
    quality: string   // 品质标签，如 "128kmp3", "flac", "臻品母带" 等
    filename: string  // 对应下载文件名，如 "M800xxxx.mp3"
    size: number      // 文件字节大小
}

// 歌曲基本信息
export interface SongInfo {
    id: number
    mid: string
    title: string
    artist: string
    album: string
    coverUrl: string
    mediaMid: string
    qualities: QualityItem[]
}

// 搜索结果完整返回
export interface SearchResponse {
    songs: SongInfo[]
    has_more: boolean
}

// 歌单基本信息
export interface PlaylistInfo {
    id: string
    name: string
    creator: string
    coverUrl: string
    songCount: number
    playCount: number
}

// 歌单接口完整返回
export interface PlaylistSongsResponse {
    playlist: PlaylistInfo
    songs: SongInfo[]
}

// 搜索建议条目（对应后端 fetch_suggestions 返回的每个 item）
export interface SearchSuggestionItem {
    id?: string
    mid?: string
    name?: string
    singer?: string
    cover?: string | null
    vid?: string          // 仅 MV 类型存在
    [key: string]: unknown
}

// 搜索建议分组数据
export interface SearchSuggestionData {
    song: SearchSuggestionItem[]
    singer: SearchSuggestionItem[]
    album: SearchSuggestionItem[]
    mv: SearchSuggestionItem[]
}

// 歌词响应
export interface LyricResponse {
    lrc: string | null;
    elrc: string | null;
    raw: string | null;
    instrumental: boolean;
}

export interface TaskRecord {
    id: string
    songId: number
    songMid: string
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
    speed?: number  // 实时下载速度 (bytes/s)，仅 downloading/paused 状态有意义
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
    saf_folder_uri?: string | null
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
    writeMetadata: false,
}

// GitHub 最新 release 信息
export interface UpdateInfo {
    tag_name: string
    name: string
    body: string
    html_url: string
    published_at: string
    prerelease: boolean
    current_version: string
}