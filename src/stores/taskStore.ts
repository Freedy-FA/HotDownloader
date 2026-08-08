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
            url: '',
            savePath: '',
            quality: task.quality,
            key: '',
            fileSize: task.fileSize,
        }).catch(console.error)
    }

    function updateTaskUrl(
        taskId: string,
        url: string,
        key: string,
        offset: number
    ) {
        invoke('update_task_url', {
            taskId,
            url,
            key,
            offset,
        }).catch(console.error)
    }

    function cancelTask(taskId: string) {
        invoke('cancel_task', { taskId }).catch(console.error)
        tasks.value = tasks.value.filter((t) => t.id !== taskId)
        saveTasks()
    }

    function removeTask(taskId: string) {
        tasks.value = tasks.value.filter((t) => t.id !== taskId)
        saveTasks()
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
                const currentIdx = QUALITY_DOWNGRADE_ORDER.indexOf(task.quality as any)
                if (currentIdx >= 0 && currentIdx < QUALITY_DOWNGRADE_ORDER.length - 1) {
                    const newQuality = QUALITY_DOWNGRADE_ORDER[currentIdx + 1]
                    task.quality = newQuality as any
                    task.retryCount = 0
                    task.errorMsg = `自动降级至 ${newQuality}`
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
        } else {
            // 未超过3次，保留 downloaded（续传用）
            // 如果错误原因是链接过期，downloaded 保留；网络错误也保留（可能已部分下载）
            task.downloaded = task.downloaded // 保持不变
        }

        task.status = 'waiting'
        if (!task.errorMsg) {
            task.errorMsg = undefined
        }
        saveTasks()
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
        updateTaskUrl,
        cancelTask,
        removeTask,
        pauseTask,
        resumeTask,
        retryTask,
        errorTask,
        setupListeners,
    }
})