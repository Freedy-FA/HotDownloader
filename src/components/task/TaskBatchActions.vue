<template>
    <div v-if="selectedCount > 0" class="batch-actions">
        <span>已选择 {{ selectedCount }} 个任务</span>
        <n-popconfirm @positive-click="$emit('clear')">
            <template #trigger>
                <n-button type="error" size="small">清除所选</n-button>
            </template>
            确认清除所选任务吗？下载中的任务不会被删除。
        </n-popconfirm>
    </div>
</template>

<script setup lang="ts">
import { NButton, NPopconfirm } from 'naive-ui'

defineProps<{
    selectedCount: number
}>()

defineEmits<{
    (e: 'clear'): void
}>()
</script>

<style scoped>
.batch-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--n-color, #fff);
    border-top: 1px solid var(--n-border-color, #e0e0e0);
    margin-top: 12px;
}
</style>