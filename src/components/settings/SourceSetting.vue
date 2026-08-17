<template>
    <n-form-item label="QQ 音乐 Cookie">
        <n-input type="textarea" :value="settingsStore.settings.qqCookie"
            @update:value="(val) => (settingsStore.settings.qqCookie = val)" :autosize="{ minRows: 3, maxRows: 6 }"
            placeholder="从 y.qq.com 复制 Cookie，需包含 uin 与 qm_keyst" />
        <template #feedback>
            <div class="source-help">
                <div>登录状态：{{ loginStatus }}</div>
                <div>
                    在浏览器打开
                    <n-a href="https://y.qq.com" target="_blank">y.qq.com</n-a>
                    并登录后，打开开发者工具 → Network，复制任意请求的 Cookie。
                    填入后即可尝试 320kmp3 等需登录音质。未登录时只能下载未加密的较低音质。
                </div>
            </div>
        </template>
    </n-form-item>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NFormItem, NInput, NA } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

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

const loginStatus = computed(() => {
    const cookie = settingsStore.settings.qqCookie?.trim() ?? ''
    if (!cookie) return '未登录（游客，无法获取 320kmp3）'
    const map = parseCookieMap(cookie)
    const uin = normalizeUin(map.uin) || normalizeUin(map.wxuin)
    const hasTicket = Boolean(map.qm_keyst || map.qqmusic_key || map.psrf_qqaccess_token)
    if (uin && hasTicket) return `已识别账号 ${uin}`
    return 'Cookie 不完整，请确认包含 uin 与 qm_keyst'
})
</script>

<style scoped>
.source-help {
    font-size: 12px;
    color: var(--n-text-color-3);
    margin-top: 4px;
    line-height: 1.6;
}
</style>
