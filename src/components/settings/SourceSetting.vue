<template>
    <n-form-item label="QQ 音乐登录">
        <div class="source-login">
            <div class="source-actions">
                <n-button type="primary" :loading="loggingIn" @click="handleLogin">
                    {{ isLoggedIn ? '重新登录' : '登录 QQ 音乐' }}
                </n-button>
                <n-button v-if="loggingIn" @click="handleCapture">我已登录，立即获取</n-button>
                <n-button v-if="isLoggedIn" @click="handleLogout">退出登录</n-button>
                <n-button text @click="showManual = !showManual">
                    {{ showManual ? '收起手动填写' : '手动粘贴 Cookie' }}
                </n-button>
            </div>
            <div class="source-help">
                <div>登录状态：{{ loginStatus }}</div>
                <div>
                    点击登录后会打开 QQ 音乐网页，用你平时的方式登录即可。
                    登录成功后会自动保存，无需自己复制 Cookie。
                </div>
            </div>
            <n-input v-if="showManual" type="textarea" :value="settingsStore.settings.qqCookie"
                @update:value="(val) => (settingsStore.settings.qqCookie = val)"
                :autosize="{ minRows: 3, maxRows: 6 }"
                placeholder="一般不用填。只有自动登录失败时，再从浏览器复制 Cookie。" />
        </div>
    </n-form-item>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { NFormItem, NInput, NButton } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

interface QqLoginCaptured {
    cookie: string
    uin: string
}

const settingsStore = useSettingsStore()
const loggingIn = ref(false)
const showManual = ref(false)
let unlisten: UnlistenFn | null = null

function parseCookieMap(cookie: string): Record<string, string> {
    const map: Record<string, string> = {}
    for (const part of cookie.split(';')) {
        const [rawKey, ...rest] = part.split('=')
        const key = rawKey?.trim().toLowerCase()
        if (!key) continue
        map[key] = rest.join('=').trim()
    }
    return map
}

function normalizeUin(raw?: string): string | null {
    if (!raw) return null
    const trimmed = raw.trim().replace(/^[oO]/, '')
    if (!trimmed || trimmed === '0' || !/^\d+$/.test(trimmed)) return null
    return trimmed
}

const detectedUin = computed(() => {
    const cookie = settingsStore.settings.qqCookie?.trim() ?? ''
    if (!cookie) return null
    const map = parseCookieMap(cookie)
    const uin = normalizeUin(map.uin) || normalizeUin(map.wxuin)
    const hasTicket = Boolean(map.qm_keyst || map.qqmusic_key || map.psrf_qqaccess_token)
    return uin && hasTicket ? uin : null
})

const isLoggedIn = computed(() => Boolean(detectedUin.value))

const loginStatus = computed(() => {
    if (loggingIn.value) return '请在弹出的窗口中登录 QQ 音乐'
    if (detectedUin.value) return `已登录账号 ${detectedUin.value}`
    const cookie = settingsStore.settings.qqCookie?.trim() ?? ''
    if (cookie) return 'Cookie 不完整，请重新登录'
    return '未登录（游客无法获取 320kmp3）'
})

function applyCaptured(captured: QqLoginCaptured) {
    settingsStore.settings.qqCookie = captured.cookie
    loggingIn.value = false
}

function notifyError(message: string) {
    if (typeof window !== 'undefined' && (window as any).$notify) {
        (window as any).$notify.error({ title: '登录失败', description: message, duration: 4000 })
    }
}

function notifySuccess(uin: string) {
    if (typeof window !== 'undefined' && (window as any).$notify) {
        (window as any).$notify.success({ title: '登录成功', description: `已登录 QQ 音乐账号 ${uin}`, duration: 3000 })
    }
}

async function handleLogin() {
    loggingIn.value = true
    try {
        await invoke('start_qq_login')
    } catch (e: any) {
        loggingIn.value = false
        notifyError(e?.message || String(e) || '打开登录窗口失败')
    }
}

async function handleCapture() {
    try {
        const captured = await invoke<QqLoginCaptured>('capture_qq_login')
        applyCaptured(captured)
        notifySuccess(captured.uin)
    } catch (e: any) {
        notifyError(e?.message || String(e) || '尚未检测到登录')
    }
}

function handleLogout() {
    settingsStore.settings.qqCookie = ''
    loggingIn.value = false
}

onMounted(async () => {
    unlisten = await listen<QqLoginCaptured>('qq-login-success', (event) => {
        applyCaptured(event.payload)
        notifySuccess(event.payload.uin)
    })
})

onUnmounted(() => {
    if (unlisten) {
        unlisten()
        unlisten = null
    }
})
</script>

<style scoped>
.source-login {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.source-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.source-help {
    font-size: 12px;
    color: var(--n-text-color-3);
    line-height: 1.6;
}
</style>
