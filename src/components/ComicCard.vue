<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { NCard } from 'naive-ui'
import { enqueueDescription } from '../descriptionQueue.ts'
import { CategoryRespData, CategorySubRespData, commands } from '../bindings.ts'
import { useStore } from '../store.ts'
import IconButton from './IconButton.vue'
import { PhDownloadSimple, PhFolderOpen } from '@phosphor-icons/vue'

const store = useStore()

const props = defineProps<{
  comicId: number
  comicTitle: string
  comicAuthor: string
  comicDescription?: string | null
  comicCategory: CategoryRespData
  comicCategorySub: CategorySubRespData
  comicDownloaded: boolean
  comicDownloadDir: string
}>()

// 列表API沒有描述時，先用本地快取，沒有再由背景佇列慢慢補抓
const description = ref<string | null | undefined>(props.comicDescription)
watch(
  () => props.comicDescription,
  (value) => {
    if (value !== undefined && value !== null) {
      description.value = value
    }
  },
)

let unmounted = false
onMounted(() => {
  if (description.value !== undefined && description.value !== null) {
    return
  }
  enqueueDescription({
    comicId: props.comicId,
    apply: (value) => {
      description.value = value
    },
    cancelled: () => unmounted,
  })
})
onUnmounted(() => {
  unmounted = true
})

async function pickComic() {
  const result = await commands.getComic(props.comicId)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  store.pickedComic = result.data
  store.currentTabName = 'chapter'
}

async function downloadComic() {
  const result = await commands.downloadComic(props.comicId)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}

async function showComicDownloadDirInFileManager() {
  if (store.config === undefined) {
    return
  }
  const result = await commands.showPathInFileManager(props.comicDownloadDir)
  if (result.status === 'error') {
    console.error(result.error)
  }
}
</script>

<template>
  <n-card content-style="padding: 0.25rem;" hoverable>
    <div class="flex">
      <img
        class="w-24 object-cover mr-4 cursor-pointer transition-transform duration-200 hover:scale-106"
        :src="`https://cdn-msp3.18comic.vip/media/albums/${comicId}_3x4.jpg`"
        alt=""
        referrerpolicy="no-referrer"
        @click="pickComic" />
      <div class="flex flex-col w-full justify-between">
        <div class="flex flex-col">
          <span
            class="font-bold text-lg line-clamp-2 cursor-pointer transition-colors duration-200 hover:text-blue-5"
            @click="pickComic">
            {{ comicTitle }}
          </span>
          <span class="text-red">作者：{{ comicAuthor }}</span>
          <span class="text-gray">分類：{{ comicCategory.title }} {{ comicCategorySub.title }}</span>
          <span
            v-if="description !== undefined && description !== null && description !== ''"
            class="text-sm text-gray-6 line-clamp-2 mt-1"
            :title="description">
            敘述：{{ description }}
          </span>
        </div>
        <div class="flex">
          <IconButton v-if="comicDownloaded" title="打開下載目錄" @click="showComicDownloadDirInFileManager">
            <PhFolderOpen :size="24" />
          </IconButton>
          <IconButton class="ml-auto" title="一鍵下載所有章節" @click="downloadComic">
            <PhDownloadSimple :size="24" />
          </IconButton>
        </div>
      </div>
    </div>
  </n-card>
</template>
