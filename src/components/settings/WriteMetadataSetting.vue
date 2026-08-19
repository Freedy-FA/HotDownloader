<template>
    <template v-if="isNarrow">
        <!-- 移动端：开关行内布局，与其他开关组件保持一致 -->
        <div class="setting-row">
            <span class="setting-label">下载完成后写入歌曲标签</span>
            <n-switch :value="settingsStore.settings.writeMetadata"
                @update:value="(val) => (settingsStore.settings.writeMetadata = val)" />
        </div>
    </template>
    <template v-else>
        <!-- 桌面端：原有表单布局 -->
        <n-form-item label="下载完成后写入歌曲标签">
            <n-switch :value="settingsStore.settings.writeMetadata"
                @update:value="(val) => (settingsStore.settings.writeMetadata = val)" />
        </n-form-item>
    </template>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { NFormItem, NSwitch } from 'naive-ui'
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