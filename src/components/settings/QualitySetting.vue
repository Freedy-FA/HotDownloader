<template>
    <n-form-item label="默认音质">
        <n-select :value="settingsStore.settings.defaultQuality"
            @update:value="(val) => (settingsStore.settings.defaultQuality = val)" :options="qualityOptions" />
    </n-form-item>
</template>

<script setup lang="ts">
import { NFormItem, NSelect } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'
import { QUALITY_DOWNGRADE_ORDER } from '../../types'

const settingsStore = useSettingsStore()

const qualityOptions = [
    { label: '每次询问', value: 'ask' as const },
    ...QUALITY_DOWNGRADE_ORDER.map((q) => ({ label: q, value: q })),
]
</script>