<template>
    <template v-if="isNarrow">
        <div class="setting-row">
            <span class="setting-label">搜索历史</span>
            <n-button size="small" @click="historyStore.clearHistory()">清除</n-button>
        </div>
    </template>
    <template v-else>
        <n-form-item label="搜索历史">
            <n-button @click="historyStore.clearHistory()">清除搜索历史</n-button>
        </n-form-item>
    </template>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { NFormItem, NButton } from 'naive-ui'
import { useHistoryStore } from '../../stores/historyStore'

const historyStore = useHistoryStore()

// 移动端判断
const isNarrow = ref(
    typeof window !== 'undefined' &&
    window.matchMedia('(max-width: 767px)').matches
)
let mediaQuery: MediaQueryList | null = null

function updateNarrow(e: MediaQueryListEvent | MediaQueryList) {
    isNarrow.value = e.matches
}

onMounted(() => {
    mediaQuery = window.matchMedia('(max-width: 767px)')
    updateNarrow(mediaQuery)
    mediaQuery.addEventListener('change', updateNarrow)
})

onUnmounted(() => {
    if (mediaQuery) {
        mediaQuery.removeEventListener('change', updateNarrow)
    }
})
</script>

<style scoped>
.setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.setting-label {
    font-size: 14px;
    color: var(--n-text-color);
}
</style>