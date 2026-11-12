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
  if (store.serialResult === undefined) {
    return 0
  }
  return Math.ceil(store.serialResult.total / PAGE_SIZE)
})

onMounted(() => getWeeklySerial(1))

async function getWeeklySerial(page: number) {
  if (loading.value) {
    return
  }

  loading.value = true
  const result = await commands.getWeeklySerial(page)
  loading.value = false
  if (result.status === 'error') {
    console.error(result.error)
    return
  }

  currentPage.value = page
  store.serialResult = result.data
}
</script>

<template>
  <div class="h-full flex flex-col gap-2">
    <div v-if="store.serialResult !== undefined" class="flex flex-col gap-row-2 overflow-auto box-border px-2 pt-2">
      <ComicCard
        v-for="comicInSerial in store.serialResult.content"
        :key="comicInSerial.id"
        :comic-id="comicInSerial.id"
        :comic-title="comicInSerial.name"
        :comic-author="comicInSerial.author"
        :comic-description="comicInSerial.description"
        :comic-category="comicInSerial.category"
        :comic-category-sub="comicInSerial.categorySub"
        :comic-downloaded="comicInSerial.isDownloaded"
        :comic-download-dir="comicInSerial.comicDownloadDir" />
    </div>

    <n-pagination
      v-if="pageCount > 0"
      class="box-border p-2 pt-0 mt-auto"
      :page-count="pageCount"
      :page="currentPage"
      @update:page="getWeeklySerial($event)" />
  </div>
</template>
