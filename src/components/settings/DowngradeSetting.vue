<template>
    <template v-if="isNarrow">
        <!-- 移动端：开关行内布局 -->
        <div class="setting-row">
            <span class="setting-label">自动降级</span>
            <n-switch :value="settingsStore.settings.autoDowngrade"
                @update:value="(val) => (settingsStore.settings.autoDowngrade = val)" />
        </div>
        <n-form-item label="降级顺序（固定）">
            <div class="downgrade-order">
                {{ downgradeOrderText }}
            </div>
        </n-form-item>
    </template>
    <template v-else>
        <!-- 桌面端：原有表单布局 -->
        <n-form-item label="自动降级">
            <n-switch :value="settingsStore.settings.autoDowngrade"
                @update:value="(val) => (settingsStore.settings.autoDowngrade = val)" />
        </n-form-item>
        <n-form-item label="降级顺序（固定）">
            <div class="downgrade-order">
                {{ downgradeOrderText }}
            </div>
        </n-form-item>
    </template>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { NFormItem, NSwitch } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'
import { QUALITY_DOWNGRADE_ORDER } from '../../types'

const settingsStore = useSettingsStore()

const downgradeOrderText = computed(() =>
    QUALITY_DOWNGRADE_ORDER.join(' > ')
)

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
.downgrade-order {
    color: var(--n-text-color-3);
    font-size: 13px;
}

/* 移动端行内布局 */
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