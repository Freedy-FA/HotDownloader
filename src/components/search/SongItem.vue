<template>
    <div class="song-item">
        <n-checkbox :checked="selected" @update:checked="$emit('toggleSelect', $event)" />
        <img v-if="song.coverUrl" :src="song.coverUrl" class="cover" alt="封面" />
        <div class="info">
            <div class="title">{{ song.title }}</div>
            <div class="subtitle">{{ song.artist }} · {{ song.album }}</div>
            <div class="quality-tags">
                <n-tag v-for="q in song.qualities.slice(0, 4)" :key="q.quality" size="tiny" :bordered="false"
                    type="info">
                    {{ q.quality }}
                </n-tag>
                <n-tag v-if="song.qualities.length > 4" size="tiny" :bordered="false" type="info">
                    +{{ song.qualities.length - 4 }}
                </n-tag>
            </div>
        </div>
        <n-button size="small" @click="$emit('download', song)">
            下载
        </n-button>
    </div>
</template>

<script setup lang="ts">
import { NCheckbox, NButton, NTag } from 'naive-ui'
import type { SongInfo } from '../../types'

defineProps<{
    song: SongInfo
    selected: boolean
}>()

defineEmits<{
    (e: 'toggleSelect', selected: boolean): void
    (e: 'download', song: SongInfo): void
}>()
</script>

<style scoped>
.song-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border: 1px solid var(--n-border-color, #eee);
    border-radius: 8px;
}

.cover {
    width: 48px;
    height: 48px;
    border-radius: 6px;
    object-fit: cover;
}

.info {
    flex: 1;
    overflow: hidden;
}

.title {
    font-size: 15px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.subtitle {
    font-size: 13px;
    color: var(--n-text-color-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.quality-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
}
</style>