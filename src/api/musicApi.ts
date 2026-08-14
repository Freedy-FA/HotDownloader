import { invoke } from '@tauri-apps/api/core'
import type { SongInfo, SearchSuggestionData } from '../types'

export async function searchSongs(
    keyword: string,
    page: number = 1,
    limit: number = 20
): Promise<SongInfo[]> {
    const json = await invoke<string>('search_songs', { keyword, page, limit })
    return JSON.parse(json) as SongInfo[]
}

export async function fetchDownloadLink(
    songId: string,
    filename: string
): Promise<{ url: string; key: string }> {
    const json = await invoke<string>('fetch_download_link', { songId, filename })
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