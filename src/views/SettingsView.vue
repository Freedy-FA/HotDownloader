<template>
    <div class="settings-view" :class="{ 'is-narrow': isNarrow }">
        <!-- 移动端：分组纵向布局 -->
        <template v-if="isNarrow">
            <div class="settings-section">
                <h2 class="section-title">基本设置</h2>
                <n-form label-placement="top">
                    <QualitySetting />
                    <DowngradeSetting />
                    <ClearHistoryButton />
                </n-form>
            </div>

            <div class="settings-section">
                <h2 class="section-title">下载设置</h2>
                <n-form label-placement="top">
                    <DirectorySetting />
                    <NamingTemplate />
                    <ConcurrencySetting />
                    <JumpToTaskSetting />
                </n-form>
            </div>
        </template>

        <!-- 桌面端：原有左右分栏表单 -->
        <template v-else>
            <n-form label-placement="left" label-width="180">
                <QualitySetting />
                <DowngradeSetting />
                <DirectorySetting />
                <NamingTemplate />
                <ConcurrencySetting />
                <JumpToTaskSetting />
                <ClearHistoryButton />
            </n-form>
        </template>

        <!-- 关于入口（始终位于页面底部） -->
        <div class="about-entry">
            <n-button text @click="goAbout">关于 HotDownloader</n-button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { NForm, NButton } from 'naive-ui'
import QualitySetting from '../components/settings/QualitySetting.vue'
import DowngradeSetting from '../components/settings/DowngradeSetting.vue'
import DirectorySetting from '../components/settings/DirectorySetting.vue'
import NamingTemplate from '../components/settings/NamingTemplate.vue'
import ConcurrencySetting from '../components/settings/ConcurrencySetting.vue'
import JumpToTaskSetting from '../components/settings/JumpToTaskSetting.vue'
import ClearHistoryButton from '../components/settings/ClearHistoryButton.vue'

const router = useRouter()

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

function goAbout() {
    router.push('/about')
}
</script>

<style scoped>
.settings-view {
    max-width: 600px;
    padding: 16px 0;
    /* 让设置页占满父容器高度，使用 flex 列布局 */
    display: flex;
    flex-direction: column;
    min-height: 100%;
}

/* 移动端移除最大宽度限制，撑满父容器 */
.settings-view.is-narrow {
    max-width: none;
}

.settings-section {
    margin-bottom: 24px;
}

.settings-section+.settings-section {
    border-top: 1px solid var(--border-color, #e0e0e0);
    padding-top: 24px;
}

.section-title {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--color-text);
}

.about-entry {
    /* 将关于入口推到底部 */
    margin-top: auto;
    padding-top: 24px;
    text-align: center;
}
</style>