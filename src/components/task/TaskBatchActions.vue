<template>
    <div v-if="selectedCount > 0" class="batch-actions">
        <span>已选择 {{ selectedCount }} 个任务</span>
        <n-popconfirm @positive-click="handleConfirm">
            <template #trigger>
                <n-button type="error" size="small">清除所选</n-button>
            </template>
            <n-space vertical :size="8">
                <span>确认清除所选任务吗？</span>
                <n-checkbox v-model:checked="deleteFile">
                    同时删除已下载或未完成的文件
                </n-checkbox>
            </n-space>
        </n-popconfirm>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NButton, NPopconfirm, NCheckbox, NSpace } from 'naive-ui'

defineProps<{
    selectedCount: number
}>()

const emit = defineEmits<{
    (e: 'clear', deleteFile: boolean): void
}>()

const deleteFile = ref(false)

function handleConfirm() {
    emit('clear', deleteFile.value)
    // 重置复选框状态，防止下次打开弹窗时保留上次勾选
    deleteFile.value = false
}
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