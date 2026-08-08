<template>
    <n-form-item label="搜索历史">
        <n-button @click="historyStore.clearHistory()">清除搜索历史</n-button>
    </n-form-item>
</template>

<script setup lang="ts">
import { NFormItem, NButton } from 'naive-ui'
import { useHistoryStore } from '../../stores/historyStore'

const historyStore = useHistoryStore()
</script>