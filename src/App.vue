<template>
  <n-config-provider :theme="theme" class="app-root">
    <n-dialog-provider class="app-root">
      <n-notification-provider>
        <NavLayout />
      </n-notification-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NConfigProvider, NDialogProvider, NNotificationProvider, darkTheme, useOsTheme } from 'naive-ui'
import NavLayout from './components/NavLayout.vue'

const theme = computed(() => (useOsTheme().value === 'dark' ? darkTheme : null))
</script>

<style scoped>
.app-root {
  height: 100%;
  color: var(--color-text);
}
</style>