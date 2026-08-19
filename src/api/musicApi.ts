import { invoke } from '@tauri-apps/api/core'
import type { SongInfo, SearchResponse, SearchSuggestionData, PlaylistSongsResponse, UpdateInfo, LyricResponse } from '../types'

export async function searchSongs(
    keyword: string,
    page: number = 1,
    limit: number = 20
): Promise<SearchResponse> {
    const json = await invoke<string>('search_songs', { keyword, page, limit })
    const parsed = JSON.parse(json) as SearchResponse
    if (Array.isArray(parsed)) {
        return { songs: parsed as unknown as SongInfo[], has_more: false }
    }
    return parsed
}

export async function fetchDownloadLink(
    songMid: string,
    filename: string
): Promise<{ url: string; key: string }> {
    const json = await invoke<string>('fetch_download_link', { songMid, filename })
    return JSON.parse(json) as { url: string; key: string }
}

// 获取热搜关键词
export async function getHotKeywords(): Promise<string[]> {
    const json = await invoke<string>('fetch_hot_keywords')
    return JSON.parse(json) as string[]
}

// 获取搜索建议
export async function fetchSuggestions(keyword: string): Promise<SearchSuggestionData> {
    const json = await invoke<string>('fetch_suggestions', { keyword })
    return JSON.parse(json) as SearchSuggestionData
}

// 获取歌单
export async function fetchPlaylistSongs(input: string): Promise<PlaylistSongsResponse> {
    const json = await invoke<string>('fetch_playlist_songs', { input })
    return JSON.parse(json) as PlaylistSongsResponse
}

// 检查 GitHub 最新版本
export async function checkForUpdate(): Promise<UpdateInfo> {
    const json = await invoke<string>('check_update')
    return JSON.parse(json) as UpdateInfo
}

/**
 * 根据 QQ 音乐歌曲 ID 获取歌词
 * @param songId QQ 音乐歌曲 ID
 */
export async function getLyricBySongId(songId: number): Promise<LyricResponse> {
    return invoke<LyricResponse>('get_lyric_by_id', { songId });
}