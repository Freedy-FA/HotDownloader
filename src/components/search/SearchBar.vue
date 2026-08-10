<template>
    <div class="search-bar">
        <n-input v-model:value="keywordModel" placeholder="搜索歌曲、歌手、专辑" clearable @keyup.enter="handleSearch"
            class="search-input" />
        <n-button type="primary" @click="handleSearch" :disabled="!keywordModel.trim()" class="search-btn">
            搜索
        </n-button>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { NInput, NButton } from 'naive-ui'

const props = defineProps<{
    keyword: string
}>()

const emit = defineEmits<{
    (e: 'update:keyword', value: string): void
    (e: 'search'): void
}>()

const keywordModel = ref(props.keyword)

// 向上同步
watch(keywordModel, (val) => {
    emit('update:keyword', val)
})

// 向下同步：当父组件 keyword 变化时更新输入框
watch(
    () => props.keyword,
    (newVal) => {
        if (newVal !== keywordModel.value) {
            keywordModel.value = newVal
        }
    }
)

function handleSearch() {
    if (keywordModel.value.trim()) {
        emit('search')
    }
}
</script>

<!-- ===== 必要的 CSS 变量（仅布局/主题，非组件颜色） ===== -->
<style>
:root {
    --search-height: 38px;
    --search-font-size: 14px;
    --search-radius: 10px;
    /* 统一圆角 */
    --search-padding: 4px 12px;
    --search-gap: 10px;

    /* 容器背景/阴影（支持深色模式） */
    --search-bg: #f5f7fa;
    --search-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
    --search-shadow-focus: 0 2px 8px rgba(0, 0, 0, 0.10);
}

@media (prefers-color-scheme: dark) {
    :root {
        --search-bg: #1e1e24;
        --search-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
        --search-shadow-focus: 0 2px 10px rgba(0, 0, 0, 0.8);
    }
}
</style>

<!-- ===== 组件样式（仅布局覆盖，颜色/圆角尽量用原生） ===== -->
<style>
.search-bar {
    display: flex;
    align-items: center;
    gap: var(--search-gap);
    margin-bottom: 16px;
    background: var(--search-bg);
    padding: var(--search-padding);
    border-radius: var(--search-radius);
    /* 外层圆角 */
    box-shadow: var(--search-shadow);
    transition: box-shadow 0.2s ease;
}

.search-bar:focus-within {
    box-shadow: var(--search-shadow-focus);
}

/* 输入框容器自动撑开 */
.search-bar .search-input {
    flex: 1;
    min-width: 0;
}

/* ---------- 让 n-input 透明，只继承外层背景 ---------- */
.search-bar .search-input .n-input {
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
    height: var(--search-height) !important;
    padding: 0 12px !important;
    /* 左右缩进，占位符不再顶左 */
    border-radius: 0 !important;
    /* 取消自身圆角，由外层统一 */
}

.search-bar .search-input .n-input-wrapper {
    background: transparent !important;
    border: none !important;
    padding: 0 !important;
    height: 100% !important;
}

/* 内部 input 零内边距，由父级控制 */
.search-bar .search-input .n-input__input {
    padding: 0 !important;
    font-size: var(--search-font-size) !important;
    height: 100% !important;
    line-height: var(--search-height) !important;
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
    color: inherit !important;
    /* 使用 Naive UI 默认文字颜色 */
}

.search-bar .search-input .n-input__input::placeholder {
    color: inherit !important;
    /* 使用 Naive UI 默认占位符颜色 */
    opacity: 0.6;
}

/* 隐藏内置边框伪元素 */
.search-bar .search-input .n-input__border,
.search-bar .search-input .n-input__state-border {
    display: none !important;
}

/* 清除按钮位置微调 */
.search-bar .search-input .n-input__clear {
    right: 4px !important;
    top: 50% !important;
    transform: translateY(-50%) !important;
}

/* ---------- 按钮：仅控制尺寸，颜色/圆角完全由 type="primary" 决定 ---------- */
.search-bar .search-btn {
    height: var(--search-height) !important;
    padding: 0 18px !important;
    font-size: var(--search-font-size) !important;
    border-radius: var(--search-radius) !important;
    /* 与外层圆角一致 */
    /* 不再覆盖背景色、文字色、边框，全由 Naive UI 原生控制 */
    /* 保留 flex 居中，防止内部文字偏移 */
    display: inline-flex !important;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
}

/* 保留悬停/禁用等状态，利用 Naive UI 的默认行为，仅微调阴影 */
.search-bar .search-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 10px rgba(24, 144, 255, 0.3);
    /* primary 蓝色的悬停阴影，可自定义或省略 */
}

.search-bar .search-btn:active:not(:disabled) {
    transform: scale(0.97);
}
</style>