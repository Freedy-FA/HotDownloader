<template>
    <div v-if="selectedCount > 0" class="batch-bar">
        <span class="selected-text">已选择 {{ selectedCount }} 首</span>
        <n-button type="primary" @click="$emit('batch-download')">
            批量下载
        </n-button>
    </div>
</template>

<script setup lang="ts">
import { NButton } from 'naive-ui'

defineProps<{
    selectedCount: number
}>()

defineEmits<{
    (e: 'batch-download'): void
}>()
</script>

<style scoped>
.batch-bar {
    position: sticky;
    bottom: 0;
    background-color: var(--bg-bottom);
    border-top: 1px solid var(--border-color);
    padding: 12px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.06);
}

.selected-text {
    font-weight: 500;
    color: var(--color-text);
}
</style>