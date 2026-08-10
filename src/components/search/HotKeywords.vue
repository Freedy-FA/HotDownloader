<template>
    <div v-if="!loading && keywords.length > 0" class="hot-keywords">
        <div class="hot-header">热搜推荐</div>
        <div class="hot-list">
            <n-tag v-for="word in keywords" :key="word" class="hot-tag" closable size="medium"
                @click="$emit('select', word)">
                {{ word }}
            </n-tag>
        </div>
    </div>
    <div v-else-if="loading" class="hot-loading">
        <n-spin size="small" />
    </div>
</template>

<script setup lang="ts">
import { NTag, NSpin } from 'naive-ui'

defineProps<{
    keywords: string[]
    loading: boolean
}>()

defineEmits<{
    (e: 'select', word: string): void
}>()
</script>

<style scoped>
.hot-keywords {
    margin-top: 16px;
}

.hot-header {
    font-size: 14px;
    font-weight: 500;
    color: var(--n-text-color-2);
    margin-bottom: 8px;
}

.hot-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.hot-tag {
    cursor: pointer;
    transition: opacity 0.2s;
}

.hot-tag:hover {
    opacity: 0.8;
}

.hot-loading {
    display: flex;
    justify-content: center;
    padding: 12px 0;
}
</style>