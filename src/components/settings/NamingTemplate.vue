<template>
    <n-form-item label="文件命名模板">
        <n-input :value="settingsStore.settings.namingTemplate"
            @update:value="(val) => (settingsStore.settings.namingTemplate = val)" placeholder="{song} - {artist}" />
        <template #feedback>
            <div class="template-help">
                可使用变量：<code>{song}</code>（歌名）、<code>{artist}</code>（歌手）、<code>{album}</code>（专辑）、<code>{quality}</code>（音质）<br />
                若替换后结果为空或仅含非法字符，将自动使用默认模板“歌名 - 歌手”
            </div>
            <div class="template-preview" v-if="exampleFilename">
                示例文件名：{{ exampleFilename }}
            </div>
        </template>
    </n-form-item>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NFormItem, NInput } from 'naive-ui'
import { useSettingsStore } from '../../stores/settingsStore'

const settingsStore = useSettingsStore()

// 用于预览的示例歌曲信息
const exampleSong = {
    song: '晴天',
    artist: '周杰伦',
    album: '叶惠美',
    quality: 'flac'
}

// 过滤非法字符的函数（与后端 sanitize_name 一致）
const sanitize = (raw: string) => raw.replace(/[\\/:*?"<>|]/g, '_')

// 根据当前模板生成示例文件名
const exampleFilename = computed(() => {
    const template = settingsStore.settings.namingTemplate || '{song} - {artist}'
    let name = template
        .replaceAll('{song}', exampleSong.song)
        .replaceAll('{artist}', exampleSong.artist)
        .replaceAll('{album}', exampleSong.album)
        .replaceAll('{quality}', exampleSong.quality)

    const sanitized = sanitize(name).trim()
    if (!sanitized) {
        const fallback = '{song} - {artist}'
            .replaceAll('{song}', exampleSong.song)
            .replaceAll('{artist}', exampleSong.artist)
            .replaceAll('{album}', exampleSong.album)
            .replaceAll('{quality}', exampleSong.quality)
        const fallbackSanitized = sanitize(fallback).trim()
        return fallbackSanitized || '未知歌曲'
    }
    return sanitized
})
</script>

<style scoped>
.template-help {
    font-size: 12px;
    color: var(--n-text-color-3);
    margin-top: 4px;
    line-height: 1.6;
}

.template-preview {
    margin-top: 6px;
    font-size: 12px;
    color: var(--n-text-color);
    background: var(--n-color-embedded);
    padding: 4px 8px;
    border-radius: 4px;
    display: inline-block;
    font-family: monospace;
}

code {
    background: rgba(127, 127, 127, 0.15);
    padding: 1px 4px;
    border-radius: 3px;
    font-family: monospace;
    font-size: 12px;
    color: inherit;
}
</style>