<template>
    <template v-if="isNarrow">
        <div class="setting-row">
            <span class="setting-label">下载歌词</span>
            <n-switch :value="settingsStore.settings.downloadLyrics"
                @update:value="(val) => (settingsStore.settings.downloadLyrics = val)" />
        </div>
    </template>
    <template v-else>
        <n-form-item label="下载歌词">
            <n-switch :value="settingsStore.settings.downloadLyrics"
                @update:value="(val) => (settingsStore.settings.downloadLyrics = val)" />
        </n-form-item>
    </template>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { NFormItem, NSwitch } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

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
