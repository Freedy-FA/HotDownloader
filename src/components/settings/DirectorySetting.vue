<template>
    <n-form-item label="下载目录">
        <n-input-group>
            <n-input :value="settingsStore.settings.downloadDir" readonly placeholder="请选择下载目录" />
            <n-button type="primary" @click="selectDirectory">选择</n-button>
        </n-input-group>
    </n-form-item>
</template>

<script setup lang="ts">
import { NFormItem, NInput, NInputGroup, NButton } from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

async function selectDirectory() {
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