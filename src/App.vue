<template>
  <n-config-provider :theme="theme" class="app-root">
    <n-dialog-provider class="app-root">
      <NavLayout />
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NConfigProvider, NDialogProvider, darkTheme, useOsTheme } from 'naive-ui'
import NavLayout from './components/NavLayout.vue'

const theme = computed(() => (useOsTheme().value === 'dark' ? darkTheme : null))
</script>

<style scoped>
.app-root {
  height: 100%;
}
</style>