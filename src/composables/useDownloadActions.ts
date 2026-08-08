import { h, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useDialog, NSelect } from 'naive-ui'
import type { Quality, SongInfo, TaskRecord, QualityItem } from '../types'
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

    /**
     * 根据期望品质和歌曲可用品质列表，返回实际可用的品质项（含 filename）
     * 若无法满足且开启自动降级，则按降级顺序选择第一个可用品质
     * 若仍无可用品质，返回 null
     */
    function resolveQualityForSong(
        song: SongInfo,
        desiredQuality: Quality
    ): QualityItem | null {
        // 直接匹配
        const direct = song.qualities.find((q) => q.quality === desiredQuality)
        if (direct) return direct

        // 自动降级
        if (settingsStore.settings.autoDowngrade) {
            for (const fallback of QUALITY_DOWNGRADE_ORDER) {
                const found = song.qualities.find((q) => q.quality === fallback)
                if (found) return found
            }
        }
        return null
    }

    /** 异步为 waiting 任务获取下载链接并更新 */
    async function fetchAndUpdateTask(taskId: string, songId: string, filename: string) {
        try {
            const { url, key } = await musicApi.fetchDownloadLink(songId, filename)
            taskStore.updateTaskUrl(taskId, url, key, 0)
        } catch (error: any) {
            taskStore.errorTask(taskId, error.message || '获取下载链接失败')
        }
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

        const resolved = resolveQualityForSong(song, quality)
        if (!resolved) {
            // 直接创建错误任务
            const taskId = generateTaskId()
            const task: TaskRecord = {
                id: taskId,
                songId: song.id,
                songTitle: song.title,
                artist: song.artist,
                album: song.album,
                coverUrl: song.coverUrl,
                mediaMid: song.mediaMid,
                filename: '',
                quality,
                status: 'error',
                errorMsg: '所选音质不可用',
                fileSize: 0,
                downloaded: 0,
                retryCount: 0,
                addedAt: Date.now(),
            }
            taskStore.addTask(task)
            return
        }

        const taskId = generateTaskId()
        const task: TaskRecord = {
            id: taskId,
            songId: song.id,
            songTitle: song.title,
            artist: song.artist,
            album: song.album,
            coverUrl: song.coverUrl,
            mediaMid: song.mediaMid,
            filename: resolved.filename,
            quality: resolved.quality as Quality, // 实际品质标签
            status: 'waiting',
            fileSize: 0,
            downloaded: 0,
            retryCount: 0,
            addedAt: Date.now(),
        }

        taskStore.addTask(task)
        // 异步获取链接，不阻塞
        fetchAndUpdateTask(taskId, song.id, resolved.filename)

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

        for (const song of songs) {
            const resolved = resolveQualityForSong(song, quality)
            if (!resolved) {
                const taskId = generateTaskId()
                const task: TaskRecord = {
                    id: taskId,
                    songId: song.id,
                    songTitle: song.title,
                    artist: song.artist,
                    album: song.album,
                    coverUrl: song.coverUrl,
                    mediaMid: song.mediaMid,
                    filename: '',
                    quality,
                    status: 'error',
                    errorMsg: '所选音质不可用',
                    fileSize: 0,
                    downloaded: 0,
                    retryCount: 0,
                    addedAt: Date.now(),
                }
                taskStore.addTask(task)
                continue
            }

            const taskId = generateTaskId()
            const task: TaskRecord = {
                id: taskId,
                songId: song.id,
                songTitle: song.title,
                artist: song.artist,
                album: song.album,
                coverUrl: song.coverUrl,
                mediaMid: song.mediaMid,
                filename: resolved.filename,
                quality: resolved.quality as Quality,
                status: 'waiting',
                fileSize: 0,
                downloaded: 0,
                retryCount: 0,
                addedAt: Date.now(),
            }
            taskStore.addTask(task)
            // 异步获取链接
            fetchAndUpdateTask(taskId, song.id, resolved.filename)
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

        // 重试时直接使用任务中保存的 filename 重新获取链接
        try {
            const { url, key } = await musicApi.fetchDownloadLink(
                currentTask.songId,
                currentTask.filename
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