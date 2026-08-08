import { h, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useDialog, NSelect } from 'naive-ui'
import type { Quality, SongInfo, TaskRecord } from '../types'
import { QUALITY_DOWNGRADE_ORDER } from '../types'
import { useSettingsStore } from '../stores/settingsStore'
import { useTaskStore } from '../stores/taskStore'
import * as musicApi from '../api/musicApi'

export function useDownloadActions() {
    const dialog = useDialog()
    const router = useRouter()
    const settingsStore = useSettingsStore()
    const taskStore = useTaskStore()

    function generateTaskId(): string {
        return Date.now().toString(36) + Math.random().toString(36).substring(2)
    }

    function askQuality(): Promise<Quality> {
        return new Promise((resolve, reject) => {
            const selected = ref<Quality>('320k')
            const d = dialog.create({
                title: '选择下载音质',
                content: () =>
                    h('div', { style: 'padding: 12px 0' }, [
                        h(NSelect, {
                            value: selected.value,
                            onUpdateValue: (val: Quality) => {
                                selected.value = val
                            },
                            options: QUALITY_DOWNGRADE_ORDER.map((q) => ({
                                label: q,
                                value: q,
                            })),
                            style: 'width: 200px',
                        }),
                    ]),
                positiveText: '确定',
                negativeText: '取消',
                onPositiveClick: () => {
                    resolve(selected.value)
                    d.destroy()
                },
                onNegativeClick: () => {
                    reject(new Error('用户取消'))
                    d.destroy()
                },
                onClose: () => {
                    reject(new Error('用户取消'))
                    d.destroy()
                },
            })
        })
    }

    async function downloadSingle(
        song: SongInfo,
        forceQuality?: Quality
    ): Promise<void> {
        let quality: Quality
        if (forceQuality) {
            quality = forceQuality
        } else if (settingsStore.settings.defaultQuality === 'ask') {
            try {
                quality = await askQuality()
            } catch {
                return
            }
        } else {
            quality = settingsStore.settings.defaultQuality
        }

        const taskId = generateTaskId()
        const task: TaskRecord = {
            id: taskId,
            songId: song.id,
            songTitle: song.title,
            artist: song.artist,
            album: song.album,
            coverUrl: song.coverUrl,
            quality,
            status: 'waiting',
            fileSize: 0,
            downloaded: 0,
            retryCount: 0,
            addedAt: Date.now(),
        }

        taskStore.addTask(task)

        try {
            const { url, key } = await musicApi.fetchDownloadLink(song.id, quality)
            taskStore.updateTaskUrl(taskId, url, key, 0)
        } catch (error: any) {
            taskStore.errorTask(taskId, error.message || '所选音质不可用')
        }

        if (settingsStore.settings.jumpToTask) {
            router.push('/task')
        }
    }

    async function batchDownload(songs: SongInfo[]): Promise<void> {
        let quality: Quality
        if (settingsStore.settings.defaultQuality === 'ask') {
            try {
                quality = await askQuality()
            } catch {
                return
            }
        } else {
            quality = settingsStore.settings.defaultQuality
        }

        const taskIds = songs.map((song) => {
            const taskId = generateTaskId()
            const task: TaskRecord = {
                id: taskId,
                songId: song.id,
                songTitle: song.title,
                artist: song.artist,
                album: song.album,
                coverUrl: song.coverUrl,
                quality,
                status: 'waiting',
                fileSize: 0,
                downloaded: 0,
                retryCount: 0,
                addedAt: Date.now(),
            }
            taskStore.addTask(task)
            return { taskId, songId: song.id }
        })

        const results = await Promise.allSettled(
            taskIds.map(({ taskId, songId }) =>
                musicApi.fetchDownloadLink(songId, quality).then(
                    (res) => ({ taskId, url: res.url, key: res.key }),
                    (err) => ({ taskId, error: err.message || '获取链接失败' })
                )
            )
        )

        for (const result of results) {
            if (result.status === 'fulfilled') {
                const value = result.value
                // 类型守卫：判断是否包含 url 字段
                if ('url' in value) {
                    const { taskId, url, key } = value
                    taskStore.updateTaskUrl(taskId, url, key, 0)
                } else {
                    taskStore.errorTask(value.taskId, value.error)
                }
            }
            // 由于内部已用 .then 捕获错误，result.status 不可能为 'rejected'
        }

        if (settingsStore.settings.jumpToTask) {
            router.push('/task')
        }
    }

    async function retryTask(taskId: string): Promise<void> {
        const task = taskStore.tasks.find((t) => t.id === taskId)
        if (!task || task.status !== 'error') return

        const canRetry = taskStore.retryTask(taskId)
        if (!canRetry) return

        const currentTask = taskStore.tasks.find((t) => t.id === taskId)
        if (!currentTask) return

        try {
            const { url, key } = await musicApi.fetchDownloadLink(
                currentTask.songId,
                currentTask.quality
            )
            taskStore.updateTaskUrl(taskId, url, key, currentTask.downloaded)
        } catch (error: any) {
            taskStore.errorTask(taskId, error.message || '重试失败')
        }
    }

    return {
        downloadSingle,
        batchDownload,
        retryTask,
    }
}