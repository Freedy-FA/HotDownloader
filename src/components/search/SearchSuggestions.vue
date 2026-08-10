<template>
    <div class="search-suggestions" v-if="hasAny">
        <!-- 单曲 -->
        <div v-if="data.song.length > 0" class="suggest-group">
            <div class="group-title">单曲</div>
            <div v-for="item in data.song" :key="item.mid" class="suggest-item" @click="$emit('select', item.name)">
                <span class="item-name">{{ item.name }}</span>
                <span class="item-singer">- {{ item.singer }}</span>
            </div>
        </div>

        <!-- 歌手 -->
        <div v-if="data.singer.length > 0" class="suggest-group">
            <div class="group-title">歌手</div>
            <div v-for="item in data.singer" :key="item.mid" class="suggest-item" @click="$emit('select', item.name)">
                <span class="item-name">{{ item.name }}</span>
            </div>
        </div>

        <!-- 专辑 -->
        <div v-if="data.album.length > 0" class="suggest-group">
            <div class="group-title">专辑</div>
            <div v-for="item in data.album" :key="item.mid" class="suggest-item" @click="$emit('select', item.name)">
                <span class="item-name">{{ item.name }}</span>
                <span class="item-singer">- {{ item.singer }}</span>
            </div>
        </div>

        <!-- MV -->
        <div v-if="data.mv.length > 0" class="suggest-group">
            <div class="group-title">MV</div>
            <div v-for="item in data.mv" :key="item.vid" class="suggest-item" @click="$emit('select', item.name)">
                <span class="item-name">{{ item.name }}</span>
                <span class="item-singer">- {{ item.singer }}</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
    data: { song: any[]; singer: any[]; album: any[]; mv: any[] }
}>()

defineEmits<{
    (e: 'select', keyword: string): void
}>()

const hasAny = computed(
    () =>
        props.data.song.length > 0 ||
        props.data.singer.length > 0 ||
        props.data.album.length > 0 ||
        props.data.mv.length > 0
)
</script>

<style scoped>
.search-suggestions {
    margin-top: 12px;
}

.suggest-group {
    margin-bottom: 12px;
}

.group-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--n-text-color-3);
    margin-bottom: 6px;
    padding-left: 4px;
}

.suggest-item {
    padding: 8px 12px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s;
    display: flex;
    align-items: baseline;
    gap: 4px;
}

.suggest-item:hover {
    background: var(--n-color-hover, rgba(0, 0, 0, 0.04));
}

.item-name {
    font-size: 14px;
    color: var(--n-text-color);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.item-singer {
    font-size: 12px;
    color: var(--n-text-color-3);
    white-space: nowrap;
}
</style>