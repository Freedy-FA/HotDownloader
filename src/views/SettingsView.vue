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
                    <WriteMetadataSetting />
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
                <WriteMetadataSetting />
                <ConcurrencySetting />
                <JumpToTaskSetting />
                <ClearHistoryButton />
            </n-form>
        </template>

        <!-- 检查更新入口（移动端与桌面端通用） -->
        <div class="check-update-entry">
            <n-button :loading="checkingUpdate" @click="handleCheckUpdate">
                检查更新
            </n-button>
            <span v-if="updateInfo && !checkingUpdate" class="update-status-text">
                {{ isNewVersion ? '发现新版本' : '已是最新版本' }}
            </span>
        </div>

        <!-- 关于入口（始终位于页面底部） -->
        <div class="about-entry">
            <n-button text @click="goAbout">关于 HotDownloader</n-button>
        </div>

        <!-- 更新信息弹窗：桌面端最大宽度 600px，移动端左右留白 16px -->
        <n-modal v-model:show="showUpdateModal" preset="card" class="update-modal"
            :title="isNewVersion ? '发现新版本' : '检查更新'" style="max-width: 600px; width: calc(100% - 32px);">
            <div v-if="updateInfo" class="update-content">
                <p class="version-line">
                    当前版本：{{ updateInfo.current_version }}
                    <template v-if="isNewVersion">
                        ｜ 最新版本：{{ updateInfo.tag_name }}
                    </template>
                </p>
                <p v-if="updateInfo.published_at" class="publish-date">
                    发布时间：{{ updateInfo.published_at }}
                </p>
                <div class="update-body">
                    <n-text class="body-label">更新内容：</n-text>
                    <div class="body-text">{{ updateInfo.body || '（无更新说明）' }}</div>
                </div>
                <div class="modal-actions">
                    <n-button type="primary" @click="showUpdateModal = false">关闭</n-button>
                    <n-button v-if="updateInfo.html_url" tag="a" :href="updateInfo.html_url" target="_blank">
                        前往发布页
                    </n-button>
                </div>
            </div>
        </n-modal>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { NForm, NButton, NModal, NText } from 'naive-ui'
import QualitySetting from '../components/settings/QualitySetting.vue'
import DowngradeSetting from '../components/settings/DowngradeSetting.vue'
import DirectorySetting from '../components/settings/DirectorySetting.vue'
import NamingTemplate from '../components/settings/NamingTemplate.vue'
import ConcurrencySetting from '../components/settings/ConcurrencySetting.vue'
import JumpToTaskSetting from '../components/settings/JumpToTaskSetting.vue'
import ClearHistoryButton from '../components/settings/ClearHistoryButton.vue'
import WriteMetadataSetting from '../components/settings/WriteMetadataSetting.vue'
import * as musicApi from '../api/musicApi'
import type { UpdateInfo } from '../types'

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

// 检查更新相关
const checkingUpdate = ref(false)
const updateInfo = ref<UpdateInfo | null>(null)
const showUpdateModal = ref(false)

// 判断是否为较新版本：简单比较去除前缀 v 后的版本号字符串
const isNewVersion = computed(() => {
    if (!updateInfo.value) return false
    const current = updateInfo.value.current_version
    const latest = updateInfo.value.tag_name.replace(/^v/, '')
    return current !== latest
})

// 显示错误通知：使用全局 $notify（NavLayout 已挂载），避免依赖未提供的 message provider
function showErrorNotification(message: string) {
    if (typeof window !== 'undefined' && (window as any).$notify) {
        (window as any).$notify.error(message)
    } else {
        console.error(message)
    }
}

async function handleCheckUpdate() {
    if (checkingUpdate.value) return
    checkingUpdate.value = true
    try {
        const info = await musicApi.checkForUpdate()
        updateInfo.value = info
        showUpdateModal.value = true
    } catch (error) {
        console.error('检查更新失败:', error)
        showErrorNotification('检查更新失败，请稍后重试')
    } finally {
        checkingUpdate.value = false
    }
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

/* 检查更新入口样式：与关于入口类似，居中 */
.check-update-entry {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    margin-top: 24px;
}

.update-status-text {
    font-size: 13px;
    color: var(--color-text-secondary);
}

.about-entry {
    /* 将关于入口推到底部 */
    margin-top: auto;
    padding-top: 24px;
    text-align: center;
}

/* 更新信息弹窗内部样式 */
.update-content {
    line-height: 1.6;
    /* 增加内容区上下空白，使弹窗不显得拥挤 */
    padding: 8px 0;
}

.version-line {
    font-size: 15px;
    font-weight: 500;
    margin-bottom: 8px;
}

.publish-date {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin-bottom: 16px;
}

.update-body {
    margin-bottom: 20px;
}

.body-label {
    font-weight: 500;
}

.body-text {
    margin-top: 4px;
    white-space: pre-wrap;
    /* 保留换行，显示更新说明中的换行 */
    color: var(--color-text);
    max-height: 300px;
    overflow-y: auto;
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
}
</style>