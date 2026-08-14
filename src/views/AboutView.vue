<template>
    <div class="about-view">
        <div class="about-card">
            <h1 class="app-title">HotDownloader</h1>
            <!-- 直接使用注入的变量，不再硬编码 -->
            <div class="app-version">版本 {{ version }}</div>
            <p class="app-description">
                基于 Tauri 2 + Vue 3 的桌面音乐下载应用，支持搜索、下载音乐歌曲。
            </p>
        </div>

        <div class="about-section">
            <h2 class="section-title">开源链接</h2>
            <n-ul class="link-list">
                <n-li>
                    <n-a href="https://github.com/lerdb/HotDownloader" target="_blank">
                        GitHub 仓库
                    </n-a>
                </n-li>
            </n-ul>
        </div>

        <div class="about-section">
            <h2 class="section-title">开放源代码许可</h2>
            <p class="license-text">
                本项目基于 <n-a href="https://www.apache.org/licenses/LICENSE-2.0" target="_blank">Apache License 2.0</n-a>
                开源。
            </p>
        </div>

        <div class="about-section">
            <h2 class="section-title">第三方组件</h2>
            <h3 class="sub-title">Rust</h3>
            <n-ul class="component-list">
                <n-li v-for="item in rustComponents" :key="item.name" class="component-item">
                    <span class="component-name">{{ item.name }}</span>
                    <span class="component-license">{{ item.license }}</span>
                </n-li>
            </n-ul>

            <h3 class="sub-title">Frontend</h3>
            <n-ul class="component-list">
                <n-li v-for="item in frontendComponents" :key="item.name" class="component-item">
                    <span class="component-name">{{ item.name }}</span>
                    <span class="component-license">{{ item.license }}</span>
                </n-li>
            </n-ul>
        </div>
    </div>
</template>

<script setup lang="ts">
import { NUl, NLi, NA } from 'naive-ui'
const version = import.meta.env.VITE_APP_VERSION

interface ComponentInfo {
    name: string
    license: string
}

const rustComponents: ComponentInfo[] = [
    { name: 'dirs', license: 'MIT / Apache-2.0' },
    { name: 'futures-util', license: 'MIT / Apache-2.0' },
    { name: 'log', license: 'MIT / Apache-2.0' },
    { name: 'reqwest', license: 'MIT / Apache-2.0' },
    { name: 'serde', license: 'MIT / Apache-2.0' },
    { name: 'serde_json', license: 'MIT / Apache-2.0' },
    { name: 'tauri', license: 'MIT / Apache-2.0' },
    { name: 'tokio', license: 'MIT' },
    { name: 'tokio-util', license: 'MIT' },
    { name: 'url', license: 'MIT / Apache-2.0' },
    { name: 'umc_qmc', license: 'Apache-2.0 OR MIT' },
]

const frontendComponents: ComponentInfo[] = [
    { name: '@tauri-apps/api', license: 'Apache-2.0 OR MIT' },
    { name: '@tauri-apps/plugin-dialog', license: 'MIT OR Apache-2.0' },
    { name: '@tauri-apps/plugin-store', license: 'MIT OR Apache-2.0' },
    { name: 'naive-ui', license: 'MIT' },
    { name: 'pinia', license: 'MIT' },
    { name: 'vue', license: 'MIT' },
    { name: 'vue-router', license: 'MIT' },
]
</script>

<style scoped>
.about-view {
    max-width: 600px;
    margin: 0 auto;
    padding: 24px 0;
    display: flex;
    flex-direction: column;
    gap: 24px;
}

.about-card {
    /* 使用全局定义的侧边栏背景色，自动适配深色模式 */
    background-color: var(--bg-sidebar);
    border-radius: 8px;
    padding: 24px;
    text-align: center;
}

.app-title {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 4px;
    color: var(--color-text);
}

.app-version {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin-bottom: 16px;
}

.app-description {
    font-size: 14px;
    color: var(--color-text-secondary);
    line-height: 1.6;
}

.about-section {
    background-color: var(--bg-sidebar);
    border-radius: 8px;
    padding: 16px 20px;
}

.section-title {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--color-text);
}

.sub-title {
    font-size: 14px;
    font-weight: 500;
    margin: 16px 0 8px;
    color: var(--color-text);
}

.link-list,
.component-list {
    list-style: none;
    padding: 0;
    margin: 0;
}

.component-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-color);
}

.component-item:last-child {
    border-bottom: none;
}

.component-name {
    font-size: 14px;
    color: var(--color-text);
}

.component-license {
    font-size: 13px;
    color: var(--color-text-secondary);
}

.license-text {
    font-size: 14px;
    color: var(--color-text-secondary);
    line-height: 1.6;
}
</style>