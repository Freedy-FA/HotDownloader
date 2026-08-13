<template>
    <n-data-table :columns="columns" :data="tasks" :row-key="(row: TaskRecord) => row.id"
        :checked-row-keys="(selectedRowKeys as any)"
        @update:checked-row-keys="(keys: any[]) => $emit('update:selectedRowKeys', keys as string[])" />
</template>

<script setup lang="ts">
import { h } from 'vue'
import { NDataTable, NTag, NProgress, NSpace } from 'naive-ui'
import type { DataTableColumn } from 'naive-ui'
import type { TaskRecord } from '../../types'
import { renderActions } from './TaskRowActions'

const isAndroid = navigator.userAgent.toLowerCase().includes('android')

const props = defineProps<{
    tasks: TaskRecord[]
    selectedRowKeys: string[]
}>()

const emit = defineEmits<{
    (e: 'update:selectedRowKeys', keys: string[]): void
    // 增加第三个参数 extra，用于传递删除文件标志等
    (e: 'action', action: string, taskId: string, extra?: Record<string, any>): void
}>()

/**
 * 格式化速度 (bytes/s) 为人类可读字符串
 */
function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec === 0) return ''
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
    let unitIndex = 0
    let value = bytesPerSec
    while (value >= 1024 && unitIndex < units.length - 1) {
        value /= 1024
        unitIndex++
    }
    return `${value.toFixed(1)} ${units[unitIndex]}`
}

/**
 * 渲染进度列
 */
function renderProgress(row: TaskRecord) {
    const percent = row.fileSize > 0 ? Math.round((row.downloaded / row.fileSize) * 100) : 0

    // 已完成状态
    if (row.status === 'completed') {
        return '100%'
    }

    // 错误状态显示错误信息
    if (row.status === 'error') {
        return row.errorMsg || ''
    }

    // 等待中
    if (row.status === 'waiting') {
        return '-'
    }

    // 下载中或暂停：显示进度条和速度
    const children = [
        h(NProgress, {
            percentage: percent,
            indicatorTextPlacement: 'inside',
            height: 20,
        })
    ]

    if (row.speed && row.speed > 0) {
        children.push(
            h('div', { style: { fontSize: '12px', color: 'var(--n-text-color-3)', marginTop: '4px' } },
                formatSpeed(row.speed)
            )
        )
    }

    return h('div', null, children)
}

const columns: DataTableColumn<TaskRecord>[] = [
    {
        type: 'selection',
        disabled: (row: TaskRecord) => row.status === 'downloading',
    },
    {
        title: '歌曲信息',
        key: 'song',
        render(row: TaskRecord) {
            return h('div', { class: 'song-info' }, [
                h('span', { class: 'song-title' }, row.songTitle || '未知歌曲'),
                h('span', { class: 'song-separator' }, ' - '),
                h('span', { class: 'song-artist' }, row.artist || '未知歌手'),
            ])
        },
    },
    {
        title: '音质',
        key: 'quality',
        width: 100,
        render(row: TaskRecord) {
            return row.quality
        },
    },
    {
        title: '状态',
        key: 'status',
        width: 100,
        render(row: TaskRecord) {
            const statusMap: Record<string, { type: string; label: string }> = {
                waiting: { type: 'info', label: '等待中' },
                downloading: { type: 'info', label: '下载中' },
                paused: { type: 'warning', label: '暂停' },
                completed: { type: 'success', label: '已完成' },
                error: { type: 'error', label: '错误' },
            }
            const s = statusMap[row.status] || { type: 'default', label: row.status }
            return h(NTag, { type: s.type as any, size: 'small' }, () => s.label)
        },
    },
    {
        title: '进度',
        key: 'progress',
        width: 200,
        render(row: TaskRecord) {
            return renderProgress(row)
        },
    },
    // 仅 Android 显示文件路径列
    ...(isAndroid ? [{
        title: '文件路径',
        key: 'filePath',
        minWidth: 200,
        render(row: TaskRecord) {
            // 紧凑显示：仅显示文件名，完整路径使用 tooltip
            const fullPath = row.filePath || ''
            const fileName = fullPath.split('/').pop() || fullPath || '-'
            return h('span', {
                style: {
                    fontSize: '12px',
                    whiteSpace: 'nowrap',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    cursor: 'default',
                },
                title: fullPath, // 原生 tooltip 显示完整路径
            }, fileName)
        },
    }] : []),
    {
        title: '操作',
        key: 'actions',
        width: 200,
        render(row: TaskRecord) {
            return h(NSpace, { justify: 'center' }, () =>
                renderActions(row, {
                    // 显式传递第三个参数，确保 extra 不被丢弃
                    emit: (action: string, taskId: string, extra?: Record<string, any>) => {
                        emit('action', action, taskId, extra)
                    },
                    isAndroid, // 传入平台标识
                })
            )
        },
    },
]
</script>

<style scoped>
.song-info {
    display: flex;
    flex-direction: row;
    align-items: baseline;
    flex-wrap: wrap;
}

.song-title {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.song-separator {
    margin: 0 4px;
    color: var(--n-text-color-3);
    font-size: 12px;
}

.song-artist {
    font-size: 12px;
    color: var(--n-text-color-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
</style>