import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
    TaskRecord,
    TaskStatus,
    DownloadProgressPayload,
    DownloadCompletedPayload,
    DownloadErrorPayload,
    DownloadLinkExpiredPayload,
} from '../types'
import { QUALITY_DOWNGRADE_ORDER } from '../types'
import { useSettingsStore } from './settingsStore'

export const useTaskStore = defineStore('tasks', () => {
    const tasks = ref<TaskRecord[]>([])

    // ---- 持久化加载 ----
    async function loadTasks() {
        try {
            const json = await invoke<string>('load_tasks')
            if (json) {
                const parsed: TaskRecord[] = JSON.parse(json)
                for (const task of parsed) {
                    if (
                        task.status === 'waiting' ||
                        task.status === 'downloading' ||
                        task.status === 'paused'
                    ) {
                        task.status = 'error'
                        task.errorMsg = '应用关闭导致中断'
                        task.downloaded = 0
                    } else if (task.status === 'error') {
                        task.downloaded = 0
                    }
                }
                tasks.value = parsed
                await saveTasks()
            }
        } catch {
            tasks.value = []
        }
    }

    async function saveTasks() {
        try {
            await invoke('save_tasks', {
                tasksJson: JSON.stringify(tasks.value),
            })
        } catch (e) {
            console.error('保存任务失败:', e)
        }
    }

    // ---- 任务操作 ----
    function addTask(task: TaskRecord) {
        tasks.value.push(task)
        saveTasks()
        invoke('add_download_task', {
            taskId: task.id,
            songId: task.songId,
            url: '',
            savePath: '',
            quality: task.quality,
            filename: task.filename,   // 传递品质文件名
            key: '',
            fileSize: task.fileSize,
            songTitle: task.songTitle,
            artist: task.artist,
            album: task.album,
        }).catch(console.error)
    }

    function cancelTask(taskId: string, deleteFile?: boolean) {
        invoke('cancel_task', { taskId, deleteFile: deleteFile ?? false })
            .catch(console.error)
        tasks.value = tasks.value.filter((t) => t.id !== taskId)
        saveTasks()
    }

    // 移除任务，不再传递 filePath，后端自行获取
    async function removeTask(taskId: string, deleteFile: boolean = false) {
        try {
            await invoke('remove_task', { taskId, deleteFile })
        } catch (e) {
            console.error('remove_task 失败:', e)
        }
        tasks.value = tasks.value.filter((t) => t.id !== taskId)
        saveTasks()
    }

    function enqueueTask(taskId: string, offset: number) {
        invoke('enqueue_task', { taskId, offset }).catch(console.error)
    }

    function pauseTask(taskId: string) {
        invoke('pause_task', { taskId }).catch(console.error)
        const task = tasks.value.find((t) => t.id === taskId)
        if (task && task.status === 'downloading') {
            task.status = 'paused'
            saveTasks()
        }
    }

    function resumeTask(taskId: string) {
        invoke('resume_task', { taskId }).catch(console.error)
        const task = tasks.value.find((t) => t.id === taskId)
        if (task && task.status === 'paused') {
            task.status = 'downloading'
            saveTasks()
        }
    }

    /**
     * 重试 / 降级逻辑
     * 返回 true 表示可继续重试（调用方需重新获取链接）
     * 返回 false 表示已永久失败，不可再重试
     */
    function retryTask(taskId: string): boolean {
        const task = tasks.value.find((t) => t.id === taskId)
        if (!task || task.status !== 'error') return false

        task.retryCount += 1

        if (task.retryCount > 3) {
            const settingsStore = useSettingsStore()
            if (settingsStore.settings.autoDowngrade) {
                const currentIdx = QUALITY_DOWNGRADE_ORDER.indexOf(task.quality)
                if (currentIdx >= 0 && currentIdx < QUALITY_DOWNGRADE_ORDER.length - 1) {
                    task.quality = QUALITY_DOWNGRADE_ORDER[currentIdx + 1]
                    task.retryCount = 0
                    task.errorMsg = `自动降级至 ${task.quality}`
                    task.downloaded = 0 // 文件不同，必须重新下载
                } else {
                    task.errorMsg = '已无更低音质可降级'
                    saveTasks()
                    return false
                }
            } else {
                task.errorMsg = '重试次数已用尽'
                saveTasks()
                return false
            }
        }

        task.status = 'waiting'
        if (!task.errorMsg) {
            task.errorMsg = undefined
        }
        saveTasks()
        enqueueTask(taskId, task.downloaded)
        return true
    }

    function errorTask(taskId: string, errorMsg: string) {
        const task = tasks.value.find((t) => t.id === taskId)
        if (task) {
            task.status = 'error'
            task.errorMsg = errorMsg
            saveTasks()
        }
    }

    // ---- 事件监听 ----
    function setupListeners() {
        listen<DownloadProgressPayload>('download-progress', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return
            task.downloaded = event.payload.downloaded
            task.fileSize = event.payload.total
            // 如果任务尚未处于 downloading，则切换为 downloading
            if (task.status !== 'downloading') {
                task.status = 'downloading'
            }
            saveTasks()
        })

        listen<DownloadCompletedPayload>('download-completed', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return
            task.status = 'completed'
            task.filePath = event.payload.final_path
            task.downloaded = task.fileSize
            saveTasks()
        })

        listen<DownloadErrorPayload>('download-error', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return
            task.status = 'error'
            task.errorMsg = event.payload.error_msg
            saveTasks()
        })

        listen<DownloadLinkExpiredPayload>('download-link-expired', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return
            task.status = 'error'
            task.errorMsg = '链接过期'
            // 保留 current_offset
            task.downloaded = event.payload.current_offset
            saveTasks()
        })
    }

    return {
        tasks,
        loadTasks,
        saveTasks,
        addTask,
        cancelTask,
        removeTask,
        enqueueTask,
        pauseTask,
        resumeTask,
        retryTask,
        errorTask,
        setupListeners,
    }
})