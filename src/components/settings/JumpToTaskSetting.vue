<template>
    <n-form-item label="添加任务后跳转至任务页">
        <n-switch :value="settingsStore.settings.jumpToTask"
            @update:value="(val) => (settingsStore.settings.jumpToTask = val)" />
    </n-form-item>
</template>

<script setup lang="ts">
import { NFormItem, NSwitch } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()
</script>