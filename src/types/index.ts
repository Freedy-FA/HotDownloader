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

/** 品质标签对应的文件名前缀与扩展名，与后端 build_qualities 保持一致 */
export const QUALITY_FILE_SPEC: Record<string, { prefix: string; ext: string }> = {
    '48kacc': { prefix: 'C200', ext: '.m4a' },
    '96kacc': { prefix: 'C400', ext: '.m4a' },
    '192kacc': { prefix: 'C600', ext: '.m4a' },
    '96kogg': { prefix: 'O4M0', ext: '.mgg' },
    '192kogg': { prefix: 'O6M0', ext: '.mgg' },
    '128kmp3': { prefix: 'M500', ext: '.mp3' },
    '320kmp3': { prefix: 'M800', ext: '.mp3' },
    ape: { prefix: 'A000', ext: '.ape' },
    flac: { prefix: 'F0M0', ext: '.mflac' },
    hires: { prefix: 'RSM1', ext: '.mflac' },
}

/** 从品质文件名提取 media_mid（4 字符前缀 + mid + 扩展名） */
export function extractMediaMid(filename: string): string | null {
    const dot = filename.lastIndexOf('.')
    const stem = dot >= 0 ? filename.slice(0, dot) : filename
    if (stem.length <= 4) return null
    return stem.slice(4)
}

/** 按品质标签重建文件名；特殊音质（杜比/臻品）无法从 mediaMid 推导，返回 null */
export function buildQualityFilename(quality: string, mediaMid: string): string | null {
    const spec = QUALITY_FILE_SPEC[quality]
    if (!spec || !mediaMid) return null
    return `${spec.prefix}${mediaMid}${spec.ext}`
}

export type Quality = string  // 不再限制字面量，兼容所有后端标签

export type TaskStatus = 'waiting' | 'downloading' | 'paused' | 'completed' | 'error'

export interface Settings {
    defaultQuality: Quality
    autoDowngrade: boolean
    downloadDir: string
    namingTemplate: string
    maxConcurrent: number
    jumpToTask: boolean
    /** 下载完成后写入同名 .lrc */
    downloadLyrics: boolean
    /** QQ 音乐网页 Cookie，用于解锁 320kmp3 等需登录音质 */
    qqCookie: string
    // 新增 SAF 文件夹 URI 和名称
    safFolderUri?: string
    safFolderName?: string
}

/** 歌曲可用的单个品质项 */
export interface QualityItem {
    quality: string   // 品质标签，如 "128kmp3", "flac", "臻品母带" 等
    filename: string  // 对应下载文件名，如 "M800xxxx.mp3"
    size: number      // 文件字节大小
}

// 歌曲基本信息
export interface SongInfo {
    id: string
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
    quality?: string
    filename?: string
    requested_quality?: string | null
}

export interface DownloadQualityChangedPayload {
    task_id: string
    requested_quality: string
    actual_quality: string
    filename: string
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
    downloadLyrics: true,
    qqCookie: '',
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