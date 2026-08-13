<template>
    <!-- 该组件使用 render 函数渲染，故 template 为空 -->
</template>

<script lang="ts">
import { defineComponent, h, type PropType } from 'vue'
import {
    NCheckbox,
    NTag,
    NProgress,
    NSpace,
    NEllipsis,
} from 'naive-ui'
import type { TaskRecord } from '../../types'
import { renderActions } from './TaskRowActions'

export default defineComponent({
    name: 'MobileTaskList',
    props: {
        tasks: {
            type: Array as PropType<TaskRecord[]>,
            required: true,
        },
        selectedRowKeys: {
            type: Array as PropType<string[]>,
            required: true,
        },
        isAndroid: {
            type: Boolean,
            default: false,
        },
    },
    emits: ['update:selectedRowKeys', 'action'],
    setup(props, { emit }) {
        // 格式化速度
        function formatSpeed(bytesPerSec: number): string {
            if (!bytesPerSec || bytesPerSec <= 0) return ''
            const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
            let unitIndex = 0
            let value = bytesPerSec
            while (value >= 1024 && unitIndex < units.length - 1) {
                value /= 1024
                unitIndex++
            }
            return `${value.toFixed(1)} ${units[unitIndex]}`
        }

        // 进度百分比
        function progressPercent(task: TaskRecord): number {
            if (task.status === 'completed') return 100
            if (task.fileSize > 0) {
                return Math.min(100, Math.round((task.downloaded / task.fileSize) * 100))
            }
            return 0
        }

        // 状态映射
        const statusMap: Record<string, { type: string; label: string }> = {
            waiting: { type: 'info', label: '等待中' },
            downloading: { type: 'info', label: '下载中' },
            paused: { type: 'warning', label: '暂停' },
            completed: { type: 'success', label: '已完成' },
            error: { type: 'error', label: '错误' },
        }

        // 操作按钮 VNode 数组
        function actionNodes(task: TaskRecord) {
            return renderActions(task, {
                emit: (action: string, taskId: string, extra?: Record<string, any>) => {
                    emit('action', action, taskId, extra)
                },
                isAndroid: props.isAndroid,
            })
        }

        // 处理多选切换
        function toggleSelection(taskId: string, checked: boolean) {
            const newKeys = new Set(props.selectedRowKeys)
            if (checked) {
                newKeys.add(taskId)
            } else {
                newKeys.delete(taskId)
            }
            emit('update:selectedRowKeys', Array.from(newKeys))
        }

        // 构建单个任务卡片
        function renderTaskCard(task: TaskRecord) {
            const status = statusMap[task.status] || { type: 'default', label: task.status }
            const checked = props.selectedRowKeys.includes(task.id)
            const disabled = task.status === 'downloading'

            const cardChildren = [
                // 头部：复选框 + 歌曲信息
                h('div', { class: 'task-card-header' }, [
                    h(NCheckbox, {
                        checked,
                        disabled,
                        'onUpdate:checked': (val: boolean) => toggleSelection(task.id, val),
                    }),
                    h('div', { class: 'task-card-song' }, [
                        h('div', { class: 'task-card-title' }, task.songTitle || '未知歌曲'),
                        h('div', { class: 'task-card-artist' }, task.artist || '未知歌手'),
                    ]),
                ]),

                // 元信息：状态、音质、速度
                h('div', { class: 'task-card-meta' }, [
                    h(NTag, { type: status.type as any, size: 'small' }, () => status.label),
                    h('span', { class: 'task-card-quality' }, task.quality),
                    task.speed && task.speed > 0
                        ? h('span', { class: 'task-card-speed' }, formatSpeed(task.speed))
                        : null,
                ]),

                // 进度条
                h('div', { class: 'task-card-progress' }, [
                    h(NProgress, {
                        percentage: progressPercent(task),
                        height: 12,
                    }),
                ]),
            ]

            // 仅安卓设备显示文件路径，位于进度条下方
            if (props.isAndroid) {
                cardChildren.push(
                    h('div', { class: 'task-card-filepath' }, [
                        h(NEllipsis, {
                            style: {
                                fontSize: '12px',
                                maxWidth: '100%',
                            },
                            expandTrigger: 'click',
                            lineClamp: 1,
                            tooltip: false,
                        }, () => task.filePath || '-')
                    ])
                )
            }

            // 操作按钮
            cardChildren.push(
                h('div', { class: 'task-card-actions' }, [
                    h(NSpace, { justify: 'end', wrap: true }, () => actionNodes(task)),
                ])
            )

            return h(
                'div',
                {
                    class: ['task-card', { 'is-selected': checked }],
                    key: task.id,
                },
                cardChildren
            )
        }

        // 返回渲染函数
        return () => {
            if (props.tasks.length === 0) {
                return h('div', { class: 'task-card-empty' }, '暂无任务')
            }
            return h(
                'div',
                { class: 'task-card-list' },
                props.tasks.map((task) => renderTaskCard(task))
            )
        }
    },
})
</script>

<style scoped>
.task-card-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 4px 0;
}

.task-card {
    background: var(--n-color, #fff);
    border: 1px solid var(--n-border-color, #e0e0e0);
    border-radius: 8px;
    padding: 12px;
    transition: border-color 0.2s;
}

.task-card.is-selected {
    border-color: var(--n-color-primary, #2080f0);
}

.task-card-header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin-bottom: 8px;
}

.task-card-song {
    flex: 1;
    min-width: 0;
    /* 允许收缩 */
}

.task-card-title {
    font-weight: 500;
    font-size: 14px;
    line-height: 1.4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--color-text, #1f1f1f);
}

.task-card-artist {
    font-size: 12px;
    color: var(--color-text-secondary, #555);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
}

.task-card-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 8px;
}

.task-card-quality {
    font-size: 12px;
    color: var(--color-text-secondary, #555);
    background: var(--n-color-embedded, #f5f5f5);
    padding: 2px 8px;
    border-radius: 4px;
}

.task-card-speed {
    font-size: 12px;
    color: var(--color-text-secondary, #555);
}

.task-card-progress {
    margin-bottom: 8px;
}

/* 新增文件路径样式 */
.task-card-filepath {
    margin-bottom: 8px;
    padding: 4px 8px;
    background: var(--n-color-embedded, #f5f5f5);
    border-radius: 4px;
    font-size: 12px;
    color: var(--color-text-secondary, #555);
}

.task-card-actions {
    /* 操作按钮容器，自动换行 */
}

.task-card-empty {
    text-align: center;
    padding: 24px 0;
    color: var(--color-text-secondary, #555);
    font-size: 14px;
}
</style>