<template>
    <div class="search-view">
        <SearchBar v-model:keyword="keyword" @search="handleSearch" />

        <SearchHistory v-if="!keyword && !hasSearched" :history="historyStore.history" @select="onHistorySelect"
            @remove="onHistoryRemove" @clear="historyStore.clearHistory" />

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
import { ref, watch } from 'vue'
import { NSpin } from 'naive-ui'
import SearchBar from '../components/search/SearchBar.vue'
import SearchHistory from '../components/search/SearchHistory.vue'
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

const historyStore = useHistoryStore()
const { downloadSingle, batchDownload } = useDownloadActions()

// 监听关键词变化：清空时重置搜索结果
watch(keyword, (newVal) => {
    if (!newVal) {
        hasSearched.value = false
        searchResults.value = []
        selectedIds.value = []
    }
})


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
</style>