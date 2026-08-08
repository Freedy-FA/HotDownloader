<template>
    <n-form-item label="自动降级">
        <n-switch :value="settingsStore.settings.autoDowngrade"
            @update:value="(val) => (settingsStore.settings.autoDowngrade = val)" />
    </n-form-item>
    <n-form-item label="降级顺序（固定）">
        <div class="downgrade-order">
            {{ downgradeOrderText }}
        </div>
    </n-form-item>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NFormItem, NSwitch } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'
import { QUALITY_DOWNGRADE_ORDER } from '../../types'

const settingsStore = useSettingsStore()

const downgradeOrderText = computed(() =>
    QUALITY_DOWNGRADE_ORDER.join(' > ')
)
</script>

<style scoped>
.downgrade-order {
    color: var(--n-text-color-3);
    font-size: 13px;
}
</style>