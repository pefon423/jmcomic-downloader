<script setup lang="ts">
import { Comic, commands } from '../../../bindings.ts'
import { useStore } from '../../../store.ts'
import { PhFilePdf, PhFileZip, PhFolderOpen } from '@phosphor-icons/vue'
import { NCheckbox } from 'naive-ui'
import IconButton from '../../../components/IconButton.vue'

const store = useStore()

const props = defineProps<{
  comic: Comic
  checkboxChecked: (comic: Comic) => boolean
  handleCheckboxClick: (comic: Comic) => void
  handleContextMenu: (comic: Comic) => void
}>()

function pickComic() {
  store.pickedComic = props.comic
  store.currentTabName = 'chapter'
}

async function exportCbz() {
  store.progressesPaneTabName = 'export'
  const result = await commands.exportCbz(props.comic)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}

async function exportPdf() {
  store.progressesPaneTabName = 'export'
  const result = await commands.exportPdf(props.comic)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}

async function showComicDownloadDirInFileManager() {
  if (store.config === undefined) {
    return
  }

  const comicDownloadDir = props.comic.comicDownloadDir

  if (comicDownloadDir === undefined || comicDownloadDir === null) {
    console.error('comicDownloadDir的值為undefined或null')
    return
  }

  const result = await commands.showPathInFileManager(comicDownloadDir)
  if (result.status === 'error') {
    console.error(result.error)
  }
}
</script>

<template>
  <div class="flex relative border border-solid rounded-md border-gray-2 p-1" @contextmenu="handleContextMenu(comic)">
    <n-checkbox
      size="large"
      class="absolute top-3 left-3 z-1"
      :checked="checkboxChecked(comic)"
      @click="handleCheckboxClick(comic)" />
    <img
      class="w-24 object-cover mr-4"
      :src="`https://cdn-msp3.18comic.vip/media/albums/${comic.id}_3x4.jpg`"
      alt=""
      :draggable="false"
      referrerpolicy="no-referrer" />
    <div class="flex flex-col w-full">
      <span
        class="font-bold text-lg line-clamp-2 cursor-pointer transition-colors duration-200 hover:text-blue-5"
        @click="pickComic">
        {{ comic.name }}
      </span>
      <span class="text-red">作者：{{ comic.author }}</span>
      <span v-if="comic.description" class="text-sm text-gray-6 line-clamp-2 mt-1" :title="comic.description">
        敘述：{{ comic.description }}
      </span>
      <div class="flex mt-auto gap-col-2">
        <IconButton title="打開下載目錄" @click="showComicDownloadDirInFileManager">
          <PhFolderOpen :size="24" />
        </IconButton>

        <IconButton class="ml-auto" title="導出cbz" @click="exportCbz">
          <PhFileZip :size="24" />
        </IconButton>

        <IconButton title="導出pdf" @click="exportPdf">
          <PhFilePdf :size="24" />
        </IconButton>
      </div>
    </div>
  </div>
</template>
