import { invoke } from '@tauri-apps/api/core'
import type { Quality, SongInfo } from '../types'

export async function searchSongs(keyword: string, page: number = 1): Promise<SongInfo[]> {
    const json = await invoke<string>('search_songs', { keyword, page })
    return JSON.parse(json) as SongInfo[]
}

export async function fetchDownloadLink(
    songId: string,
    quality: Quality
): Promise<{ url: string; key: string }> {
    const json = await invoke<string>('fetch_download_link', { songId, quality })
    return JSON.parse(json) as { url: string; key: string }
}