<template>
    <div class="search-view">
        <SearchBar v-model:keyword="keyword" @search="handleSearch" />

        <!-- 输入非空且未搜索：显示搜索建议 -->
        <SearchSuggestions v-if="showSuggestions" :data="suggestions" @select="onSuggestionSelect" />

        <!-- 输入为空且未搜索：显示历史与热搜 -->
        <div v-if="!keyword && !hasSearched">
            <SearchHistory :history="historyStore.history" @select="onHistorySelect" @remove="onHistoryRemove"
                @clear="historyStore.clearHistory" />
            <HotKeywords :keywords="hotKeywords" :loading="hotLoading" @select="onHotClick" />
        </div>

        <!-- 加载中 -->
        <div v-if="loading" class="loading-wrapper">
            <n-spin size="medium" />
        </div>

        <!-- 搜索结果列表（已搜索完毕） -->
        <SearchResultList v-if="hasSearched && !loading" :songs="searchResults" v-model:selectedIds="selectedIds"
            @download="onSingleDownload" @retry="handleSearch" />

        <BatchDownloadBar v-if="selectedIds.length > 0" :selectedCount="selectedIds.length"
            @batch-download="onBatchDownload" />
    </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'
import { NSpin } from 'naive-ui'
import SearchBar from '../components/search/SearchBar.vue'
import SearchHistory from '../components/search/SearchHistory.vue'
import HotKeywords from '../components/search/HotKeywords.vue'
import SearchSuggestions from '../components/search/SearchSuggestions.vue'
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

// ==================== 搜索建议相关 ====================
const suggestions = ref<{ song: any[]; singer: any[]; album: any[]; mv: any[] }>({
    song: [],
    singer: [],
    album: [],
    mv: [],
})

let abortController: AbortController | null = null
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// 是否显示建议：关键词非空且未进入搜索结果页
const showSuggestions = computed(() => {
    return keyword.value.trim() !== '' && !hasSearched.value
})

// 防抖请求建议
watch(keyword, (newVal) => {
    if (debounceTimer) {
        clearTimeout(debounceTimer)
    }
    if (abortController) {
        abortController.abort() // 取消上次请求
    }

    const term = newVal.trim()
    if (!term) {
        suggestions.value = { song: [], singer: [], album: [], mv: [] }
        return
    }

    debounceTimer = setTimeout(async () => {
        const controller = new AbortController()
        abortController = controller
        try {
            // musicApi.fetchSuggestions 需要改造以支持 AbortController，这里暂时直接调用
            // 如果后端不支持 abort，至少避免旧请求覆盖新结果
            const res = await musicApi.fetchSuggestions(term)
            if (!controller.signal.aborted) {
                suggestions.value = res
            }
        } catch {
            // 忽略错误，建议列表清空
            if (!controller.signal.aborted) {
                suggestions.value = { song: [], singer: [], album: [], mv: [] }
            }
        } finally {
            if (abortController === controller) {
                abortController = null
            }
        }
    }, 300)
})

// 点击建议项
function onSuggestionSelect(word: string) {
    keyword.value = word
    handleSearch()
}
// ==================== 建议逻辑结束 ====================

// 关键词清空时重置状态
watch(keyword, (newVal) => {
    if (!newVal) {
        hasSearched.value = false
        searchResults.value = []
        selectedIds.value = []
        suggestions.value = { song: [], singer: [], album: [], mv: [] }
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

// 搜索历史点击
function onHistorySelect(term: string) {
    keyword.value = term
    handleSearch()
}

function onHistoryRemove(term: string) {
    historyStore.removeHistoryItem(term)
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
    /* 防止底部导航遮挡 */
    min-height: 100%;
    padding-bottom: 0;
}

.loading-wrapper {
    display: flex;
    justify-content: center;
    padding: 40px 0;
}
</style>