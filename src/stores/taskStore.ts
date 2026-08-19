import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
    TaskRecord,
    DownloadProgressPayload,
    DownloadCompletedPayload,
    DownloadErrorPayload,
    DownloadLinkExpiredPayload,
} from '../types'
import { QUALITY_DOWNGRADE_ORDER } from '../types'
import { useSettingsStore } from './settingsStore'

export const useTaskStore = defineStore('tasks', () => {
    const tasks = ref<TaskRecord[]>([])

    // 简易全局通知辅助函数
    const notify = () => (window as any).$notify

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
        } catch (e: any) {
            console.error('加载任务失败:', e)
            notify()?.error({ title: '加载任务失败', description: e?.message || String(e), duration: 3000 })
        }
    }

    async function saveTasks() {
        try {
            await invoke('save_tasks', {
                tasksJson: JSON.stringify(tasks.value),
            })
        } catch (e: any) {
            console.error('保存任务失败:', e)
            notify()?.error({ title: '保存任务失败', description: e?.message || String(e), duration: 3000 })
        }
    }

    // ---- 任务操作 ----
    function addTask(task: TaskRecord) {
        tasks.value.push(task)
        saveTasks()
        invoke('add_download_task', {
            taskId: task.id,
            songId: task.songId,
            songMid: task.songMid,
            url: '',
            savePath: '',
            quality: task.quality,
            filename: task.filename,   // 传递品质文件名
            key: '',
            fileSize: task.fileSize,
            songTitle: task.songTitle,
            artist: task.artist,
            album: task.album,
            coverUrl: task.coverUrl,
        }).catch((e: any) => {
            console.error('添加任务失败:', e)
            notify()?.error({ title: '添加任务失败', description: e?.message || String(e), duration: 3000 })
        })
    }

    function cancelTask(taskId: string, deleteFile?: boolean) {
        invoke('cancel_task', { taskId, deleteFile: deleteFile ?? false })
            .catch((e: any) => {
                console.error('取消任务失败:', e)
                notify()?.error({ title: '取消任务失败', description: e?.message || String(e), duration: 3000 })
            })
        tasks.value = tasks.value.filter((t) => t.id !== taskId)
        saveTasks()
    }

    // 移除任务，统一由后端处理文件删除（包括 SAF 模式）
    async function removeTask(taskId: string, deleteFile: boolean = false) {
        try {
            await invoke('remove_task', { taskId, deleteFile })
        } catch (e: any) {
            console.error('移除任务失败:', e)
            notify()?.error({ title: '移除任务失败', description: e?.message || String(e), duration: 3000 })
        }
        tasks.value = tasks.value.filter((t) => t.id !== taskId)
        await saveTasks()
    }

    // 现在 enqueueTask 等待后端结果，并处理错误
    async function enqueueTask(taskId: string, offset: number): Promise<boolean> {
        try {
            await invoke('enqueue_task', { taskId, offset })
            return true
        } catch (e: any) {
            console.error('重新入队失败:', e)
            notify()?.error({ title: '重新入队失败', description: e?.message || String(e), duration: 3000 })
            // 将任务恢复为 error 状态
            const task = tasks.value.find((t) => t.id === taskId)
            if (task && task.status === 'waiting') {
                task.status = 'error'
                task.errorMsg = '启动下载失败，请稍后重试'
                await saveTasks()
            }
            return false
        }
    }

    function pauseTask(taskId: string) {
        invoke('pause_task', { taskId })
            .catch((e: any) => {
                console.error('暂停任务失败:', e)
                notify()?.error({ title: '暂停任务失败', description: e?.message || String(e), duration: 3000 })
            })
        const task = tasks.value.find((t) => t.id === taskId)
        if (task && task.status === 'downloading') {
            task.status = 'paused'
            saveTasks()
        }
    }

    function resumeTask(taskId: string) {
        invoke('resume_task', { taskId })
            .catch((e: any) => {
                console.error('恢复任务失败:', e)
                notify()?.error({ title: '恢复任务失败', description: e?.message || String(e), duration: 3000 })
            })
        const task = tasks.value.find((t) => t.id === taskId)
        if (task && task.status === 'paused') {
            task.status = 'downloading'
            saveTasks()
        }
    }

    /**
     * 重试 / 降级逻辑，等待 enqueue 结果
     * 返回 true 表示可继续重试（调用方需重新获取链接）
     * 返回 false 表示已永久失败，不可再重试
     */
    async function retryTask(taskId: string): Promise<boolean> {
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
                    await saveTasks()
                    return false
                }
            } else {
                task.errorMsg = '重试次数已用尽'
                await saveTasks()
                return false
            }
        }

        task.status = 'waiting'
        // 清除旧错误信息
        task.errorMsg = undefined
        await saveTasks()

        const success = await enqueueTask(taskId, task.downloaded)
        if (!success) {
            // enqueueTask 内部已将状态设为 error，但需确保错误信息正确
            const t = tasks.value.find((t) => t.id === taskId)
            if (t && t.status === 'error' && !t.errorMsg) {
                t.errorMsg = '启动下载失败'
            }
            return false
        }
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
            task.speed = event.payload.speed   // 保存速度
            // 如果任务尚未处于 downloading，则切换为 downloading
            if (task.status !== 'downloading') {
                task.status = 'downloading'
            }
            saveTasks()
        })

        // 监听下载完成
        listen<DownloadCompletedPayload>('download-completed', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return

            task.status = 'completed'
            // SAF 模式下 final_path 已经是完整 URI，无需额外处理
            task.filePath = event.payload.final_path
            task.downloaded = task.fileSize

            saveTasks()
            // 成功通知
            notify()?.success({
                title: '下载完成',
                description: `歌曲“${task.songTitle}”已下载完成`,
                duration: 3000
            })
        })

        listen<DownloadErrorPayload>('download-error', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return
            task.status = 'error'
            task.errorMsg = event.payload.error_msg
            saveTasks()
            // 弹出错误通知
            notify()?.error({
                title: '下载失败',
                description: `歌曲“${task.songTitle}”错误：${event.payload.error_msg}`,
                duration: 3000
            })
        })

        listen<DownloadLinkExpiredPayload>('download-link-expired', (event) => {
            const task = tasks.value.find((t) => t.id === event.payload.task_id)
            if (!task) return
            task.status = 'error'
            task.errorMsg = '链接过期'
            task.downloaded = event.payload.current_offset
            saveTasks()
            // 弹出链接过期通知
            notify()?.warning({
                title: '链接过期',
                description: `歌曲“${task.songTitle}”下载链接过期，请稍后重试`,
                duration: 3000
            })
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