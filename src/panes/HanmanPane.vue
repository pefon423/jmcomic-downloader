<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { commands } from '../bindings.ts'
import { NPagination } from 'naive-ui'
import ComicCard from '../components/ComicCard.vue'
import { useStore } from '../store.ts'

const store = useStore()

const PAGE_SIZE = 80

const loading = ref<boolean>(false)
const currentPage = ref<number>(1)

const pageCount = computed<number>(() => {
  if (store.hanmanResult === undefined) {
    return 0
  }
  return Math.ceil(store.hanmanResult.total / PAGE_SIZE)
})

onMounted(() => getLatestHanman(1))

async function getLatestHanman(page: number) {
  if (loading.value) {
    return
  }

  loading.value = true
  const result = await commands.getLatestHanman(page)
  loading.value = false
  if (result.status === 'error') {
    console.error(result.error)
    return
  }

  currentPage.value = page
  store.hanmanResult = result.data
}
</script>

<template>
  <div class="h-full flex flex-col gap-2">
    <div v-if="store.hanmanResult !== undefined" class="flex flex-col gap-row-2 overflow-auto box-border px-2 pt-2">
      <ComicCard
        v-for="comicInHanman in store.hanmanResult.content"
        :key="comicInHanman.id"
        :comic-id="comicInHanman.id"
        :comic-title="comicInHanman.name"
        :comic-author="comicInHanman.author"
        :comic-description="comicInHanman.description"
        :comic-category="comicInHanman.category"
        :comic-category-sub="comicInHanman.categorySub"
        :comic-downloaded="comicInHanman.isDownloaded"
        :comic-download-dir="comicInHanman.comicDownloadDir" />
    </div>

    <n-pagination
      v-if="pageCount > 0"
      class="box-border p-2 pt-0 mt-auto"
      :page-count="pageCount"
      :page="currentPage"
      @update:page="getLatestHanman($event)" />
  </div>
</template>
