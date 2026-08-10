<template>
    <div class="search-view">
        <SearchBar v-model:keyword="keyword" @search="handleSearch" />

        <div v-if="!keyword && !hasSearched">
            <SearchHistory :history="historyStore.history" @select="onHistorySelect" @remove="onHistoryRemove"
                @clear="historyStore.clearHistory" />
            <HotKeywords :keywords="hotKeywords" :loading="hotLoading" @select="onHotClick" />
        </div>

        <div v-if="loading" class="loading-wrapper">
            <n-spin size="medium" />
        </div>

        <SearchResultList v-if="hasSearched && !loading" :songs="searchResults" v-model:selectedIds="selectedIds"
            @download="onSingleDownload" />

        <BatchDownloadBar v-if="selectedIds.length > 0" :selectedCount="selectedIds.length"
            @batch-download="onBatchDownload" />
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { NSpin } from 'naive-ui'
import SearchBar from '../components/search/SearchBar.vue'
import SearchHistory from '../components/search/SearchHistory.vue'
import HotKeywords from '../components/search/HotKeywords.vue'
import SearchResultList from '../components/search/SearchResultList.vue'
import BatchDownloadBar from '../components/search/BatchDownloadBar.vue'
import { useHistoryStore } from '../stores/historyStore'
import { useDownloadActions } from '../composables/useDownloadActions'
import * as musicApi from '../api/musicApi'
import type { SongInfo } from '../types'

const keyword = ref('')
const searchResults = ref<SongInfo[]>([])
const selectedIds = ref<string[]>([])
const loading = ref(false)
const hasSearched = ref(false)

// 热搜
const hotKeywords = ref<string[]>([])
const hotLoading = ref(false)

const historyStore = useHistoryStore()
const { downloadSingle, batchDownload } = useDownloadActions()

// 清空关键词时重置页面
watch(keyword, (newVal) => {
    if (!newVal) {
        hasSearched.value = false
        searchResults.value = []
        selectedIds.value = []
    }
})

// 获取热搜
async function fetchHotKeywords() {
    hotLoading.value = true
    try {
        hotKeywords.value = await musicApi.getHotKeywords()
    } catch {
        hotKeywords.value = []
    } finally {
        hotLoading.value = false
    }
}

onMounted(() => {
    fetchHotKeywords()
})

// 热搜点击
function onHotClick(word: string) {
    keyword.value = word
    handleSearch()
}

async function handleSearch() {
    const term = keyword.value.trim()
    if (!term) return

    loading.value = true
    hasSearched.value = true
    selectedIds.value = []

    try {
        searchResults.value = await musicApi.searchSongs(term, 1)
        historyStore.addHistory(term)
    } catch (error) {
        console.error('搜索失败:', error)
        searchResults.value = []
    } finally {
        loading.value = false
    }
}

// 搜索历史点击
function onHistorySelect(term: string) {
    keyword.value = term
    handleSearch()
}

function onHistoryRemove(term: string) {
    historyStore.removeHistoryItem(term)
}

function onSingleDownload(song: SongInfo) {
    downloadSingle(song)
}

function onBatchDownload() {
    const songs = searchResults.value.filter((s) => selectedIds.value.includes(s.id))
    if (songs.length > 0) {
        batchDownload(songs)
    }
}
</script>

<style scoped>
.search-view {
    display: flex;
    flex-direction: column;
    height: 100%;
}

.loading-wrapper {
    display: flex;
    justify-content: center;
    padding: 40px 0;
}

/* 热搜区域样式 */
.hot-section {
    margin-top: 16px;
}

.hot-header {
    font-size: 14px;
    font-weight: 500;
    color: var(--n-text-color-2);
    margin-bottom: 8px;
}

.hot-loading {
    display: flex;
    justify-content: center;
    padding: 12px 0;
}

.hot-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}

.hot-tag {
    cursor: pointer;
    transition: opacity 0.2s;
}

.hot-tag:hover {
    opacity: 0.8;
}
</style>