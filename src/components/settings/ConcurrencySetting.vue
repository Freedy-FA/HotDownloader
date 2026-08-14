<template>
    <template v-if="isNarrow">
        <div class="setting-row">
            <span class="setting-label">同时下载数</span>
            <n-input-number :value="settingsStore.settings.maxConcurrent"
                @update:value="(val) => (settingsStore.settings.maxConcurrent = val ?? 1)" :min="1" :max="10" step="1"
                button-placement="both" />
        </div>
    </template>
    <template v-else>
        <n-form-item label="同时下载数">
            <n-input-number :value="settingsStore.settings.maxConcurrent"
                @update:value="(val) => (settingsStore.settings.maxConcurrent = val ?? 1)" :min="1" :max="10" step="1"
                button-placement="both" />
        </n-form-item>
    </template>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { NFormItem, NInputNumber } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

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