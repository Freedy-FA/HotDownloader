<template>
    <div class="nav-layout" :class="{ 'is-narrow': isNarrow }">
        <!-- 宽屏左侧垂直导航 -->
        <aside v-if="!isNarrow" class="sidebar">
            <n-menu :value="currentRoute" :options="menuOptions" @update:value="handleMenuClick" />
        </aside>

        <!-- 内容区域 -->
        <main class="main-content" :class="{ 'has-bottom-nav': isNarrow }">
            <router-view v-slot="{ Component }">
                <keep-alive>
                    <component :is="Component" />
                </keep-alive>
            </router-view>
        </main>

        <!-- 窄屏底部水平导航（固定底部，永远居中） -->
        <footer v-if="isNarrow" class="bottom-nav">
            <div class="bottom-nav-inner">
                <n-menu :value="currentRoute" :options="menuOptions" mode="horizontal"
                    @update:value="handleMenuClick" />
            </div>
        </footer>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { NMenu, type MenuOption } from 'naive-ui'
import { useCloseGuard } from '../composables/useCloseGuard'

const router = useRouter()
const route = useRoute()

// 在 n-dialog-provider 内部调用，确保 useDialog 正常工作
useCloseGuard()

const isNarrow = ref(false)

let mediaQuery: MediaQueryList | null = null

function updateNarrow(e: MediaQueryListEvent | MediaQueryList) {
    isNarrow.value = e.matches
}

onMounted(() => {
    mediaQuery = window.matchMedia('(max-width: 767px)')
    updateNarrow(mediaQuery)
    mediaQuery.addEventListener('change', updateNarrow)
})

onUnmounted(() => {
    if (mediaQuery) {
        mediaQuery.removeEventListener('change', updateNarrow)
    }
})

const currentRoute = computed(() => route.path)

const menuOptions: MenuOption[] = [
    {
        label: '搜索',
        key: '/search'
    },
    {
        label: '任务',
        key: '/task'
    },
    {
        label: '设置',
        key: '/settings'
    },
]

function handleMenuClick(key: string) {
    if (key !== route.path) {
        router.push(key)
    }
}
</script>

<style scoped>
/* 布局整体 */
.nav-layout {
    display: flex;
    height: 100%;
}

.nav-layout.is-narrow {
    flex-direction: column;
}

/* 侧边栏：使用自定义背景变量 */
.sidebar {
    width: 160px;
    flex-shrink: 0;
    border-right: 1px solid var(--border-color);
    padding: 12px 0;
    background-color: var(--bg-sidebar);
}

/* 主内容区背景 */
.main-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    background-color: var(--bg-content);
}

.main-content.has-bottom-nav {
    padding-bottom: 60px;
}

/* 底部导航：使用自定义背景变量 */
.bottom-nav {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 1000;
    height: 56px;
    border-top: 1px solid var(--border-color);
    background-color: var(--bg-bottom);
}

/* 居中容器 */
.bottom-nav-inner {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
}

/* 穿透样式强制菜单项居中 */
.bottom-nav-inner :deep(.n-menu) {
    justify-content: center;
}

.bottom-nav-inner :deep(.n-menu .n-menu-item) {
    flex: none;
}
</style>