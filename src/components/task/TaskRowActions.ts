import { h, ref, type VNode } from 'vue'
import { NButton, NPopconfirm, NCheckbox, NSpace } from 'naive-ui'
import type { TaskRecord } from '../../types'

export interface TaskActionContext {
    emit: (action: string, taskId: string, extra?: Record<string, any>) => void
}

export function renderActions(
    task: TaskRecord,
    context: TaskActionContext
): VNode[] {
    const { emit } = context
    const taskId = task.id
    const nodes: VNode[] = []

    // 等待中：取消（可能文件尚未创建，但用户可选择同时删除潜在的空文件）
    if (task.status === 'waiting') {
        nodes.push(
            createCancelWithDeletePopconfirm(task, emit, taskId)
        )
    }

    // 下载中：暂停（不弹窗询问删除）
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

    // 暂停：恢复、取消（取消时询问删除文件）
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
            createCancelWithDeletePopconfirm(task, emit, taskId)
        )
    }

    // 错误：重试、删除（删除任务记录，不再提示删除文件）
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

    // 已完成：打开文件位置、删除（删除任务记录，不删除已下载文件）
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

/**
 * 创建带“删除文件”选项的取消确认弹窗
 */
function createCancelWithDeletePopconfirm(
    task: TaskRecord,
    emit: TaskActionContext['emit'],
    taskId: string
) {
    // 使用响应式 ref，确保复选框状态实时更新
    const deleteFile = ref(false)

    return h(
        NPopconfirm,
        {
            onPositiveClick: () => {
                emit('cancel', taskId, { deleteFile: deleteFile.value })
            },
        },
        {
            trigger: () =>
                h(NButton, { size: 'small', type: 'warning' }, () => '取消'),
            default: () => {
                return h(NSpace, { vertical: true, size: 'small' }, () => [
                    h('span', {}, '确定取消该任务吗？'),
                    h(
                        NCheckbox,
                        {
                            checked: deleteFile.value,
                            'onUpdate:checked': (val: boolean) => {
                                deleteFile.value = val
                            },
                        },
                        () => '同时删除未下载完成的文件'
                    ),
                ])
            },
        }
    )
}