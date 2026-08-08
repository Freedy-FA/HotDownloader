<template>
    <n-form-item label="同时下载数">
        <n-input-number :value="settingsStore.settings.maxConcurrent"
            @update:value="(val) => (settingsStore.settings.maxConcurrent = val ?? 1)" :min="1" :max="10" step="1"
            button-placement="both" />
    </n-form-item>
</template>

<script setup lang="ts">
import { NFormItem, NInputNumber } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()
</script>