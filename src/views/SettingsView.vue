<template>
    <div class="settings-view">
        <n-form label-placement="left" label-width="180">
            <QualitySetting />
            <DowngradeSetting />
            <DirectorySetting />
            <NamingTemplate />
            <ConcurrencySetting />
            <JumpToTaskSetting />
            <ClearHistoryButton />
        </n-form>
    </div>
</template>

<script setup lang="ts">
import { NForm } from 'naive-ui'
import QualitySetting from '../components/settings/QualitySetting.vue'
import DowngradeSetting from '../components/settings/DowngradeSetting.vue'
import DirectorySetting from '../components/settings/DirectorySetting.vue'
import NamingTemplate from '../components/settings/NamingTemplate.vue'
import ConcurrencySetting from '../components/settings/ConcurrencySetting.vue'
import JumpToTaskSetting from '../components/settings/JumpToTaskSetting.vue'
import ClearHistoryButton from '../components/settings/ClearHistoryButton.vue'
</script>

<style scoped>
.settings-view {
    max-width: 600px;
    padding: 16px 0;
}
</style>