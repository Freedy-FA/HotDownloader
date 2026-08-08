<template>
  <n-config-provider>
    <n-dialog-provider>
      <NavLayout />
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { NConfigProvider, NDialogProvider } from 'naive-ui'
import NavLayout from './components/NavLayout.vue'
</script>