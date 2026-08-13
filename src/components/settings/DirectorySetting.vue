<template>
    <n-form-item label="下载目录">
        <template v-if="!isAndroid">
            <n-input-group>
                <n-input :value="settingsStore.settings.downloadDir" readonly placeholder="请选择下载目录" />
                <n-button type="primary" @click="selectDirectory">选择</n-button>
            </n-input-group>
        </template>
        <template v-else>
            <n-select :value="settingsStore.settings.downloadDir" :options="presetDirs" @update:value="onSelectDir"
                placeholder="请选择下载目录" />
        </template>
    </n-form-item>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NFormItem, NInput, NInputGroup, NButton, NSelect } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

// 简单平台判断
const isAndroid = ref(navigator.userAgent.toLowerCase().includes('android'))

// Android 端预设公共目录
const presetDirs = [
    { label: '下载 (Download)', value: '/storage/emulated/0/Download' },
    { label: '音乐 (Music)', value: '/storage/emulated/0/Music' },
    { label: '电影 (Movies)', value: '/storage/emulated/0/Movies' },
    { label: '图片 (Pictures)', value: '/storage/emulated/0/Pictures' },
    { label: '文档 (Documents)', value: '/storage/emulated/0/Documents' },
]

function onSelectDir(value: string | null) {
    if (value) {
        settingsStore.settings.downloadDir = value
    }
}

// 桌面端目录选择
async function selectDirectory() {
    // 仅桌面端调用
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: '选择下载目录',
        })
        if (selected && typeof selected === 'string') {
            settingsStore.settings.downloadDir = selected
        }
    } catch (error) {
        console.error('选择目录失败:', error)
    }
}
</script>