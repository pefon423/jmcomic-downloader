<script setup lang="ts">
import { useStore } from '../../../store.ts'
import { NCheckbox, NInputGroup, NInputGroupLabel, NInputNumber, NSelect, NTooltip } from 'naive-ui'

const store = useStore()

const exportSkipModeOptions = [
  { label: '不跳過，每次都重新導出', value: 'None' },
  { label: '跳過已存在的檔案', value: 'SkipExisting' },
  { label: '跳過曾導出過的章節', value: 'SkipExported' },
]
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col">
    <div class="flex gap-1 items-center">
      <n-input-group class="w-70">
        <n-input-group-label size="small">創建pdf并發數</n-input-group-label>
        <n-input-number
          class="w-full"
          v-model:value="store.config.createPdfConcurrency"
          size="small"
          :min="1"
          :parse="(x: string) => Number(x)" />
      </n-input-group>
      <n-tooltip placement="top" trigger="hover">
        <div>
          <span>在</span>
          <span class="rounded bg-gray-500 px-1">章節詳情</span>
          <span>里手動勾選導出的PDF一律不會自動合并</span>
        </div>
        <div>
          <span>只有在</span>
          <span class="rounded bg-gray-500 px-1">本地庫存</span>
          <span>里直接導出整部作品為PDF</span>
        </div>
        <div>
          <span>且導出策略不為</span>
          <span class="rounded bg-gray-500 px-1">跳過曾導出過的章節</span>
          <span>時才會觸發自動合并</span>
        </div>
        <template #trigger>
          <n-checkbox class="ml-4 w-fit" v-model:checked="store.config.enableMergePdf">創建完成后自動合并</n-checkbox>
        </template>
      </n-tooltip>
    </div>

    <n-tooltip placement="top" trigger="hover">
      <div>
        <span>只影響</span>
        <span class="rounded bg-gray-500 px-1">本地庫存</span>
        <span>里直接導出整部作品時的行為</span>
      </div>
      <div>
        <span>在</span>
        <span class="rounded bg-gray-500 px-1">章節詳情</span>
        <span>里手動勾選導出時一律以</span>
        <span class="rounded bg-gray-500 px-1">不跳過，每次都重新導出</span>
        <span>處理</span>
      </div>
      <template #trigger>
        <n-input-group class="mt-2 w-fit">
          <n-input-group-label size="small">導出策略</n-input-group-label>
          <n-select
            v-model:value="store.config.exportSkipMode"
            :options="exportSkipModeOptions"
            size="small"
            class="w-50" />
        </n-input-group>
      </template>
    </n-tooltip>
  </div>
</template>
獁