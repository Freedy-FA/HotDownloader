<template>
    <div class="search-result-list" v-if="songs.length > 0">
        <div class="list-header">
            <n-checkbox :checked="isAllSelected" :indeterminate="isIndeterminate" @update:checked="toggleAll">
                全选
            </n-checkbox>
            <span class="count-text">已选 {{ selectedIds.length }} / {{ songs.length }} 首</span>
        </div>

        <div class="song-items">
            <SongItem v-for="song in songs" :key="song.id" :song="song" :selected="selectedIds.includes(song.id)"
                @toggle-select="(val) => toggleSelect(song.id, val)" @download="(song) => $emit('download', song)" />
        </div>
    </div>
    <n-empty v-else description="暂无搜索结果" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NCheckbox, NEmpty } from 'naive-ui'
import type { SongInfo } from '../../types'
import SongItem from './SongItem.vue'

const props = defineProps<{
    songs: SongInfo[]
    selectedIds: string[]
}>()

const emit = defineEmits<{
    (e: 'update:selectedIds', ids: string[]): void
    (e: 'download', song: SongInfo): void
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
    color: var(--n-text-color-3);
}

.song-items {
    display: flex;
    flex-direction: column;
    gap: 8px;
}
</style>