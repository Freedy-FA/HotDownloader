<template>
    <div class="task-view">
        <TaskTabs v-model:activeTab="activeTab" :counts="tabCounts" />

        <TaskTable :tasks="filteredTasks" :selectedRowKeys="selectedRowKeys"
            @update:selectedRowKeys="selectedRowKeys = $event" @action="handleAction" />

        <TaskBatchActions :selectedCount="selectedRowKeys.length" @clear="handleBatchClear" />
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTaskStore } from '../stores/taskStore'
import { useDownloadActions } from '../composables/useDownloadActions'
import TaskTabs from '../components/task/TaskTabs.vue'
import TaskTable from '../components/task/TaskTable.vue'
import TaskBatchActions from '../components/task/TaskBatchActions.vue'
import type { TaskRecord, TaskStatus } from '../types'

const taskStore = useTaskStore()
const { retryTask } = useDownloadActions()

const activeTab = ref('all')
const selectedRowKeys = ref<string[]>([])

const tabCounts = computed(() => {
    const counts = {
        total: 0,
        waiting: 0,
        downloading: 0,
        paused: 0,
        completed: 0,
        error: 0,
    }
    for (const task of taskStore.tasks) {
        counts.total++
        if (task.status === 'waiting') counts.waiting++
        else if (task.status === 'downloading') counts.downloading++
        else if (task.status === 'paused') counts.paused++
        else if (task.status === 'completed') counts.completed++
        else if (task.status === 'error') counts.error++
    }
    return counts
})

const filteredTasks = computed(() => {
    const all = taskStore.tasks.filter((t) => {
        // 显式读取进度相关字段，建立响应式依赖
        void t.downloaded;
        void t.fileSize;
        return activeTab.value === 'all' || t.status === activeTab.value;
    });
    return all;
})

async function handleAction(action: string, taskId: string, extra?: any) {
    switch (action) {
        case 'cancel':
            taskStore.cancelTask(taskId, extra?.deleteFile === true)
            break
        case 'pause':
            taskStore.pauseTask(taskId)
            break
        case 'resume':
            taskStore.resumeTask(taskId)
            break
        case 'retry':
            await retryTask(taskId)
            break
        case 'remove':
            taskStore.removeTask(taskId)
            break
        case 'open-location': {
            const task = taskStore.tasks.find((t) => t.id === taskId)
            if (task?.filePath) {
                try {
                    await invoke('open_file_location', { path: task.filePath })
                } catch (e) {
                    console.error('打开文件位置失败:', e)
                }
            }
            break
        }
    }
    // 清除相关选中状态
    selectedRowKeys.value = selectedRowKeys.value.filter((id) => id !== taskId)
}

async function handleBatchClear() {
    const ids = selectedRowKeys.value.slice()
    for (const taskId of ids) {
        const task = taskStore.tasks.find((t) => t.id === taskId)
        if (!task) continue
        if (
            task.status === 'waiting' ||
            task.status === 'downloading' ||
            task.status === 'paused'
        ) {
            // 批量清除暂时默认不删除文件，可通过扩展实现
            taskStore.cancelTask(taskId, false)
        } else {
            taskStore.removeTask(taskId)
        }
    }
    selectedRowKeys.value = []
}
</script>

<style scoped>
.task-view {
    display: flex;
    flex-direction: column;
    height: 100%;
}
</style>