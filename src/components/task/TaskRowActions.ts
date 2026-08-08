import { h, type VNode } from 'vue'
import { NButton, NPopconfirm } from 'naive-ui'
import type { TaskRecord } from '../../types'

export interface TaskActionContext {
    emit: (action: string, taskId: string) => void
}

export function renderActions(
    task: TaskRecord,
    context: TaskActionContext
): VNode[] {
    const { emit } = context
    const taskId = task.id
    const nodes: VNode[] = []

    // 等待中：取消
    if (task.status === 'waiting') {
        nodes.push(
            h(
                NPopconfirm,
                {
                    onPositiveClick: () => emit('cancel', taskId),
                },
                {
                    trigger: () =>
                        h(NButton, { size: 'small', type: 'warning' }, () => '取消'),
                    default: () => '确定取消该任务吗？',
                }
            )
        )
    }

    // 下载中：暂停
    if (task.status === 'downloading') {
        nodes.push(
            h(
                NButton,
                {
                    size: 'small',
                    type: 'warning',
                    onClick: () => emit('pause', taskId),
                },
                () => '暂停'
            )
        )
    }

    // 暂停：恢复、取消
    if (task.status === 'paused') {
        nodes.push(
            h(
                NButton,
                {
                    size: 'small',
                    type: 'primary',
                    onClick: () => emit('resume', taskId),
                },
                () => '恢复'
            )
        )
        nodes.push(
            h(
                NPopconfirm,
                {
                    onPositiveClick: () => emit('cancel', taskId),
                },
                {
                    trigger: () =>
                        h(NButton, { size: 'small', type: 'warning' }, () => '取消'),
                    default: () => '确定取消该任务吗？',
                }
            )
        )
    }

    // 错误：重试、删除
    if (task.status === 'error') {
        const isRetriable =
            task.errorMsg !== '重试次数已用尽' &&
            task.errorMsg !== '已无更低音质可降级'

        nodes.push(
            h(
                NButton,
                {
                    size: 'small',
                    type: 'primary',
                    disabled: !isRetriable,
                    onClick: () => {
                        if (isRetriable) emit('retry', taskId)
                    },
                },
                () => '重试'
            )
        )
        nodes.push(
            h(
                NPopconfirm,
                {
                    onPositiveClick: () => emit('remove', taskId),
                },
                {
                    trigger: () =>
                        h(NButton, { size: 'small', type: 'error' }, () => '删除'),
                    default: () => '确定删除该任务记录吗？',
                }
            )
        )
    }

    // 已完成：打开文件位置、删除
    if (task.status === 'completed') {
        nodes.push(
            h(
                NButton,
                {
                    size: 'small',
                    onClick: () => emit('open-location', taskId),
                },
                () => '打开文件位置'
            )
        )
        nodes.push(
            h(
                NPopconfirm,
                {
                    onPositiveClick: () => emit('remove', taskId),
                },
                {
                    trigger: () =>
                        h(NButton, { size: 'small', type: 'error' }, () => '删除'),
                    default: () => '确定删除该任务记录吗？',
                }
            )
        )
    }

    return nodes
}