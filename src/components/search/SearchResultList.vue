<template>
    <div class="search-result-list">
        <template v-if="songs.length > 0">
            <div class="list-header">
                <n-checkbox :checked="isAllSelected" :indeterminate="isIndeterminate" @update:checked="toggleAll">
                    全选
                </n-checkbox>
                <span class="count-text">已选 {{ selectedIds.length }} / {{ songs.length }} 首</span>
            </div>

            <div class="song-items">
                <SongItem v-for="song in songs" :key="song.id" :song="song" :selected="selectedIds.includes(song.id)"
                    @toggle-select="(val) => toggleSelect(song.id, val)"
                    @download="(song) => $emit('download', song)" />
            </div>
        </template>

        <div v-else class="empty-result">
            <n-empty description="暂无搜索结果" />
            <div class="retry-wrapper">
                <n-button type="primary" @click="$emit('retry')">重试</n-button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NCheckbox, NEmpty, NButton } from 'naive-ui'
import type { SongInfo } from '../../types'
import SongItem from './SongItem.vue'

const props = defineProps<{
    songs: SongInfo[]
    selectedIds: string[]
}>()

const emit = defineEmits<{
    (e: 'update:selectedIds', ids: string[]): void
    (e: 'download', song: SongInfo): void
    (e: 'retry'): void          // 新增重试事件
}>()

const isAllSelected = computed(
    () => props.songs.length > 0 && props.selectedIds.length === props.songs.length
)

const isIndeterminate = computed(
    () => props.selectedIds.length > 0 && props.selectedIds.length < props.songs.length
)

function toggleAll(checked: boolean) {
    if (checked) {
        emit('update:selectedIds', props.songs.map((s) => s.id))
    } else {
        emit('update:selectedIds', [])
    }
}

function toggleSelect(songId: string, selected: boolean) {
    let newIds: string[]
    if (selected) {
        newIds = [...props.selectedIds, songId]
    } else {
        newIds = props.selectedIds.filter((id) => id !== songId)
    }
    emit('update:selectedIds', newIds)
}
</script>

<style scoped>
.list-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
}

.count-text {
    font-size: 13px;
    color: var(--color-text-secondary);
}

.song-items {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.empty-result {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px 0;
}

.retry-wrapper {
    margin-top: 16px;
    text-align: center;
}
</style>