import { createApp } from 'vue'
import { createPinia } from 'pinia'
import naive from 'naive-ui'
import App from './App.vue'
import router from './router'
import { useSettingsStore } from './stores/settingsStore'
import { useHistoryStore } from './stores/historyStore'
import { useTaskStore } from './stores/taskStore'
import './style.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(naive)

async function init() {
    const settingsStore = useSettingsStore()
    const historyStore = useHistoryStore()
    const taskStore = useTaskStore()

    // 并行加载持久化数据，即使失败也继续挂载
    try {
        await Promise.all([
            settingsStore.loadSettings(),
            historyStore.loadHistory(),
            taskStore.loadTasks(),
        ])
    } catch (e) {
        console.error('加载持久化数据失败，使用默认值:', e)
    }

    // 设置下载事件监听（内部已处理错误）
    try {
        taskStore.setupListeners()
    } catch (e) {
        console.error('注册下载事件监听失败:', e)
    }
}

init().finally(() => {
    app.mount('#app')
})