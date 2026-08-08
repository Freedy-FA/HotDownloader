import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Settings } from '../types'
import { DEFAULT_SETTINGS } from '../types'

export const useSettingsStore = defineStore('settings', () => {
    const settings = ref<Settings>({ ...DEFAULT_SETTINGS })

    async function loadSettings() {
        try {
            const json = await invoke<string>('load_settings')
            if (json) {
                const parsed = JSON.parse(json)
                settings.value = { ...DEFAULT_SETTINGS, ...parsed }
            }
        } catch {
            // 使用默认设置
        }
    }

    async function getDefaultDownloadDir() {
        try {
            const dir = await invoke<string>('get_default_download_dir')
            settings.value.downloadDir = dir
        } catch {
            // 保持现有值
        }
    }

    // 防抖持久化
    let debounceTimer: ReturnType<typeof setTimeout> | null = null
    watch(
        settings,
        () => {
            if (debounceTimer) clearTimeout(debounceTimer)
            debounceTimer = setTimeout(() => {
                invoke('save_settings', {
                    settingsJson: JSON.stringify(settings.value),
                }).catch(console.error)
            }, 500)
        },
        { deep: true }
    )

    // 并发数实时同步到后端
    watch(
        () => settings.value.maxConcurrent,
        (val) => {
            invoke('set_max_concurrent', { max: val }).catch(console.error)
        }
    )

    return {
        settings,
        loadSettings,
        getDefaultDownloadDir,
    }
})