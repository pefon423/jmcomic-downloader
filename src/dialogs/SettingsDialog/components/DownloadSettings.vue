<script setup lang="ts">
import { ref, watch } from 'vue'
import { useStore } from '../../../store.ts'
import { NCheckbox, NInput, NRadio, NRadioGroup, NTooltip, useMessage } from 'naive-ui'

const store = useStore()

const message = useMessage()

const dirFmt = ref<string>(store.config?.dirFmt ?? '')

watch([() => store.config?.apiDomainMode, () => store.config?.customApiDomain], () => {
  message.warning('切換線路后可能需要重新登入')
})
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col">
    <span class="font-bold mt-2">下載格式</span>
    <n-radio-group v-model:value="store.config.downloadFormat">
      <n-tooltip placement="top" trigger="hover">
        <template #trigger>
          <n-radio value="Jpeg">jpg</n-radio>
        </template>
        1. 有損
        <span class="text-red">(肉眼看不出)</span>
        <br />
        2. 檔案體積小
        <br />
        4. 寬高的上限為65534
        <span class="text-red">(某些條漫可能會超過這個上限導致報錯)</span>
        <br />
        3. 編碼速度最快
        <br />
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <template #trigger>
          <n-radio value="Png">png</n-radio>
        </template>
        1. 無損
        <br />
        2. 檔案體積大
        <span class="text-red">(約為jpg的5倍)</span>
        <br />
        3. 編碼速度最慢
        <br />
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <template #trigger>
          <n-radio value="Webp">webp</n-radio>
        </template>
        1. 無損
        <br />
        2. 檔案體積大
        <span class="text-red">(約為jpg的4倍)</span>
        <br />
        3. 寬高的上限為16383
        <span class="text-red">(某些條漫可能會超過這個上限導致報錯)</span>
        <br />
        4. 編碼速度較慢
        <br />
      </n-tooltip>
    </n-radio-group>

    <span class="font-bold mt-2">下載目錄格式</span>
    <n-tooltip placement="top" trigger="hover" :width="550">
      <div>
        可以用斜杠
        <span class="rounded bg-gray-500 px-1 text-white">/</span>
        來分隔目錄層級
      </div>
      <div class="text-orange">至少要有兩個層級，最后一層存放章節元數據，倒數第二層存放漫畫元數據</div>
      <div class="font-semibold mt-2">可用字段：</div>
      <div class="grid grid-cols-2">
        <div>
          <span class="rounded bg-gray-500 px-1">comic_id</span>
          <span class="ml-2">漫畫ID</span>
        </div>
        <div>
          <span class="rounded bg-gray-500 px-1">chapter_id</span>
          <span class="ml-2">章節ID</span>
        </div>
        <div>
          <span class="rounded bg-gray-500 px-1">comic_title</span>
          <span class="ml-2">漫畫標題</span>
        </div>
        <div>
          <span class="rounded bg-gray-500 px-1">chapter_title</span>
          <span class="ml-2">章節標題</span>
        </div>
        <div>
          <span class="rounded bg-gray-500 px-1">author</span>
          <span class="ml-2">作者</span>
        </div>
        <div>
          <span class="rounded bg-gray-500 px-1">order</span>
          <span class="ml-2">章節在漫畫里對應的序號</span>
        </div>
      </div>
      <div class="font-semibold mt-2">例如格式</div>
      <div class="bg-gray-200 rounded-md p-1 text-black w-fit">
        {author}/[{author}] {comic_title}({comic_id})/{order} - {chapter_title}
      </div>
      <div class="font-semibold">下載《驀然回首》第1話會產生三層資料夾，分別是</div>
      <div class="flex gap-1 text-black">
        <span class="bg-gray-200 rounded-md px-2 w-fit">藤本樹, 藤本タツキ</span>
        <span class="rounded bg-gray-500 px-1 text-white">/</span>
        <span class="bg-gray-200 rounded-md px-2 w-fit">[藤本樹, 藤本タツキ] 驀然回首(384524)</span>
        <span class="rounded bg-gray-500 px-1 text-white">/</span>
        <span class="bg-gray-200 rounded-md px-2 w-fit">1 - 第1話</span>
      </div>
      <template #trigger>
        <n-input
          v-model:value="dirFmt"
          size="small"
          @blur="store.config.dirFmt = dirFmt"
          @keydown.enter="store.config.dirFmt = dirFmt" />
      </template>
    </n-tooltip>

    <span class="font-bold mt-2">其他</span>
    <n-checkbox class="w-fit" v-model:checked="store.config.shouldDownloadCover">下載封面</n-checkbox>
    <n-tooltip placement="top" trigger="hover">
      <template #trigger>
        <n-checkbox class="w-fit" v-model:checked="store.config.zipAfterDownload">下載后轉成zip并刪除目錄</n-checkbox>
      </template>
      每個章節下載完成后，將章節目錄打包成同名zip，并刪除原目錄
      <br />
      更新庫存時，zip中的章節同樣會被識別為已下載
      <br />
      <span class="text-red">已轉成zip的章節暫時無法導出PDF/CBZ</span>
    </n-tooltip>
  </div>
</template>
