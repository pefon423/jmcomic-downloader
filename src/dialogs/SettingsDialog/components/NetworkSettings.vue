<script setup lang="ts">
import { ref, watch } from 'vue'
import { useStore } from '../../../store.ts'
import { NInput, NInputGroup, NInputGroupLabel, NInputNumber, NRadioButton, NRadioGroup, useMessage } from 'naive-ui'

const store = useStore()

const message = useMessage()

const proxyHost = ref<string>(store.config?.proxyHost ?? '')
const customApiDomain = ref<string>(store.config?.customApiDomain ?? '')

watch([() => store.config?.apiDomainMode, () => store.config?.customApiDomain], () => {
  message.warning('切換線路后可能需要重新登入')
})
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col">
    <span class="font-bold">下載速度</span>
    <div class="flex flex-col gap-1">
      <div class="flex gap-1">
        <n-input-group class="w-35%">
          <n-input-group-label size="small">章節并發數</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.chapterConcurrency"
            size="small"
            @update:value="message.warning('對章節并發數的修改需要重啟才能生效')"
            :min="1"
            :parse="(x: string) => Number(x)" />
        </n-input-group>
        <n-input-group class="w-65%">
          <n-input-group-label size="small">每個章節下載完成后休息</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.chapterDownloadIntervalSec"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)" />
          <n-input-group-label size="small">秒</n-input-group-label>
        </n-input-group>
      </div>
      <div class="flex gap-1">
        <n-input-group class="w-35%">
          <n-input-group-label size="small">圖片并發數</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.imgConcurrency"
            size="small"
            @update-value="message.warning('對圖片并發數的修改需要重啟才能生效')"
            :min="1"
            :parse="(x: string) => Number(x)" />
        </n-input-group>
        <n-input-group class="w-65%">
          <n-input-group-label size="small">每張圖片下載完成后休息</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.imgDownloadIntervalSec"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)" />
          <n-input-group-label size="small">秒</n-input-group-label>
        </n-input-group>
      </div>
      <n-input-group>
        <n-input-group-label size="small">下載整個收藏夾時，每處理完一個收藏夾中的漫畫后休息</n-input-group-label>
        <n-input-number
          class="w-full"
          v-model:value="store.config.downloadAllFavoritesIntervalSec"
          size="small"
          :min="0"
          :parse="(x: string) => Number(x)" />
        <n-input-group-label size="small">秒</n-input-group-label>
      </n-input-group>
      <n-input-group>
        <n-input-group-label size="small">更新庫存時，每處理完一個已下載的漫畫后休息</n-input-group-label>
        <n-input-number
          class="w-full"
          v-model:value="store.config.updateDownloadedComicsIntervalSec"
          size="small"
          :min="0"
          :parse="(x: string) => Number(x)" />
        <n-input-group-label size="small">秒</n-input-group-label>
      </n-input-group>
    </div>

    <span class="font-bold mt-2">API域名</span>
    <n-radio-group v-model:value="store.config.apiDomainMode" size="small">
      <n-radio-button value="Domain1">線路1</n-radio-button>
      <n-radio-button value="Domain2">線路2</n-radio-button>
      <n-radio-button value="Domain3">線路3</n-radio-button>
      <n-radio-button value="Domain4">線路4</n-radio-button>
      <n-radio-button value="Domain5">線路5</n-radio-button>
      <n-radio-button value="Custom">自定義</n-radio-button>
    </n-radio-group>
    <n-input-group v-if="store.config.apiDomainMode === 'Custom'" class="mt-1">
      <n-input-group-label size="small">自定義API域名</n-input-group-label>
      <n-input
        v-model:value="customApiDomain"
        size="small"
        placeholder=""
        @blur="store.config.customApiDomain = customApiDomain"
        @keydown.enter="store.config.customApiDomain = customApiDomain" />
    </n-input-group>

    <span class="font-bold mt-2">代理類型</span>
    <n-radio-group v-model:value="store.config.proxyMode" size="small">
      <n-radio-button value="System">系統代理</n-radio-button>
      <n-radio-button value="NoProxy">直連</n-radio-button>
      <n-radio-button value="Custom">自定義</n-radio-button>
    </n-radio-group>
    <n-input-group v-if="store.config.proxyMode === 'Custom'" class="mt-1">
      <n-input-group-label size="small">http://</n-input-group-label>
      <n-input
        v-model:value="proxyHost"
        size="small"
        placeholder=""
        @blur="store.config.proxyHost = proxyHost"
        @keydown.enter="store.config.proxyHost = proxyHost" />
      <n-input-group-label size="small">:</n-input-group-label>
      <n-input-number
        v-model:value="store.config.proxyPort"
        size="small"
        placeholder=""
        :parse="(x: string) => parseInt(x)" />
    </n-input-group>
  </div>
</template>
