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

const props = defineProps<{
    tasks: TaskRecord[]
    selectedRowKeys: string[]
}>()

const emit = defineEmits<{
    (e: 'update:selectedRowKeys', keys: string[]): void
    (e: 'action', action: string, taskId: string): void
}>()

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
                h('span', { class: 'song-artist' }, row.artist || '未知歌手'),
            ])
        },
    },
    {
        title: '音质',
        key: 'quality',
        width: 80,
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
        width: 180,
        render(row: TaskRecord) {
            if (row.status === 'completed') {
                return '100%'
            }
            if (row.status === 'downloading' || row.status === 'paused') {
                const percent = row.fileSize > 0 ? Math.round((row.downloaded / row.fileSize) * 100) : 0
                return h(NProgress, {
                    percentage: percent,
                    indicatorTextPlacement: 'inside',
                    height: 20,
                })
            }
            if (row.status === 'error') {
                return row.errorMsg || ''
            }
            return '-'
        },
    },
    {
        title: '操作',
        key: 'actions',
        width: 200,
        render(row: TaskRecord) {
            return h(NSpace, { justify: 'center' }, () =>
                renderActions(row, {
                    emit: (action: string, taskId: string) => emit('action', action, taskId),
                })
            )
        },
    },
]
</script>

<style scoped>
.song-info {
    display: flex;
    flex-direction: column;
}

.song-title {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.song-artist {
    font-size: 12px;
    color: var(--n-text-color-3);
}
</style>