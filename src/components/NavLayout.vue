<template>
    <div class="nav-layout" :class="{ 'is-narrow': isNarrow }">
        <!-- 宽屏左侧垂直导航 -->
        <aside v-if="!isNarrow" class="sidebar">
            <n-menu :value="currentRoute" :options="menuOptions" @update:value="handleMenuClick" />
        </aside>

        <!-- 内容区域 -->
        <main class="main-content">
            <router-view v-slot="{ Component }">
                <keep-alive>
                    <component :is="Component" />
                </keep-alive>
            </router-view>
        </main>

        <!-- 窄屏底部水平导航 -->
        <footer v-if="isNarrow" class="bottom-nav">
            <n-menu :value="currentRoute" :options="menuOptions" mode="horizontal" @update:value="handleMenuClick" />
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
        key: '/search',
    },
    {
        label: '任务',
        key: '/task',
    },
    {
        label: '设置',
        key: '/settings',
    },
]

function handleMenuClick(key: string) {
    if (key !== route.path) {
        router.push(key)
    }
}
</script>

<style scoped>
.nav-layout {
    display: flex;
    height: 100%;
}

.nav-layout.is-narrow {
    flex-direction: column;
}

.sidebar {
    width: 160px;
    flex-shrink: 0;
    border-right: 1px solid var(--n-border-color, #e0e0e0);
    padding: 12px 0;
}

.main-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
}

.bottom-nav {
    border-top: 1px solid var(--n-border-color, #e0e0e0);
    padding: 4px 0;
    display: flex;
    justify-content: center;
}
</style>