import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useDialog } from 'naive-ui'
import { useTaskStore } from '../stores/taskStore'

export function useCloseGuard() {
  const dialog = useDialog()
  const taskStore = useTaskStore()
  const appWindow = getCurrentWindow()
  let unlisten: (() => void) | undefined

  onMounted(async () => {
    unlisten = await appWindow.onCloseRequested(async (event) => {
      const activeTasks = taskStore.tasks.filter(
        (t) =>
          t.status === 'waiting' ||
          t.status === 'downloading' ||
          t.status === 'paused'
      )

      if (activeTasks.length === 0) {
        return // 允许关闭
      }

      event.preventDefault()

      const confirmed = await new Promise<boolean>((resolve) => {
        dialog.warning({
          title: '确认退出',
          content: `有 ${activeTasks.length} 个下载任务尚未完成，退出后进度将丢失。确认退出吗？`,
          positiveText: '确认退出',
          negativeText: '取消',
          onPositiveClick: () => resolve(true),
          onNegativeClick: () => resolve(false),
          onClose: () => resolve(false),
        })
      })

      if (confirmed) {
        // 先取消监听，避免 destroy 时再次触发
        if (unlisten) unlisten()
        await appWindow.destroy()
      }
    })
  })

  onUnmounted(() => {
    if (unlisten) unlisten()
  })
}