<template>
    <div class="search-bar">
        <n-input v-model:value="keywordModel" placeholder="搜索歌曲、歌手、专辑" clearable @keyup.enter="handleSearch">
            <template #suffix>
                <n-button type="primary" @click="handleSearch" :disabled="!keywordModel.trim()">
                    搜索
                </n-button>
            </template>
        </n-input>
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

watch(keywordModel, (val) => {
    emit('update:keyword', val)
})

function handleSearch() {
    if (keywordModel.value.trim()) {
        emit('search')
    }
}
</script>

<style scoped>
.search-bar {
    margin-bottom: 16px;
}
</style>