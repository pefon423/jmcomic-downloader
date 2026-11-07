<script setup lang="ts">
import { MessageReactive, NButton, NPopconfirm, useMessage } from 'naive-ui'
import { computed, onMounted, ref } from 'vue'
import { commands, events, UpdateDownloadedComicsEvent } from '../../../bindings.ts'
import { useStore } from '../../../store.ts'

const message = useMessage()

const store = useStore()

const popConfirmShowing = ref<boolean>(false)

const rejectCooldown = ref<number>(0)
const rejectButtonDisabled = computed(() => rejectCooldown.value > 0)

const countdownInterval = ref<ReturnType<typeof setInterval>>(setInterval(() => {}, 1000))

type ProgressData = Extract<UpdateDownloadedComicsEvent, { event: 'CreateDownloadTasksStart' }>['data'] & {
  progressMessage: MessageReactive
}

const progresses = ref<Map<number, ProgressData>>(new Map())
let updateMessage: MessageReactive | undefined

onMounted(async () => {
  await events.updateDownloadedComicsEvent.listen(async ({ payload: updateEvent }) => {
    if (updateEvent.event === 'GetComicStart') {
      updateMessage = message.loading(`正在獲取已下載漫畫的最新數據`, { duration: 0 })
    } else if (updateEvent.event === 'GetComicProgress' && updateMessage !== undefined) {
      const { current, total } = updateEvent.data
      updateMessage.content = `正在獲取已下載漫畫的最新數據(${current}/${total})`
    } else if (updateEvent.event === 'CreateDownloadTasksStart') {
      const { comicId, comicTitle, current, total } = updateEvent.data
      progresses.value.set(comicId, {
        comicId,
        comicTitle,
        current,
        total,
        progressMessage: message.loading(
          () => {
            const progressData = progresses.value.get(comicId)
            if (progressData === undefined) return ''
            return `${progressData.comicTitle} 正在創建下載任務(${progressData.current}/${progressData.total})`
          },
          { duration: 0 },
        ),
      })
    } else if (updateEvent.event === 'CreateDownloadTaskProgress') {
      const { comicId, current } = updateEvent.data
      const progressData = progresses.value.get(comicId)
      if (progressData) {
        progressData.current = current
      }
    } else if (updateEvent.event === 'CreateDownloadTasksEnd' && updateMessage !== undefined) {
      const { comicId } = updateEvent.data
      const progressData = progresses.value.get(comicId)
      if (progressData) {
        progressData.progressMessage.type = 'success'
        progressData.progressMessage.content = `${progressData.comicTitle} 創建下載任務完成(${progressData.current}/${progressData.total})`
        setTimeout(() => {
          progressData.progressMessage.destroy()
          progresses.value.delete(comicId)
        }, 3000)
      }
    } else if (updateEvent.event === 'GetComicEnd' && updateMessage !== undefined) {
      updateMessage.type = 'success'
      updateMessage.content = '已獲取所有已下載漫畫的最新數據，并為需要更新的章節創建了下載任務'
      setTimeout(() => {
        updateMessage?.destroy()
        updateMessage = undefined
      }, 5000)
    }
  })
})

async function agree() {
  if (store.config === undefined) {
    return
  }

  // 1秒下載5張
  store.config.imgDownloadIntervalSec = Math.max(1, Math.floor(store.config.imgConcurrency / 5))
  store.config.chapterDownloadIntervalSec = Math.min(10, Math.floor(store.config.imgConcurrency * 3))

  popConfirmShowing.value = false

  const result = await commands.updateDownloadedComics()
  if (result.status === 'error') {
    console.error(result.error)
    updateMessage?.destroy()
    progresses.value.forEach((progress) => {
      progress.progressMessage.destroy()
    })
    progresses.value.clear()
    return
  }
}

async function reject() {
  popConfirmShowing.value = false
  const result = await commands.updateDownloadedComics()
  if (result.status === 'error') {
    console.error(result.error)
    updateMessage?.destroy()
    progresses.value.forEach((progress) => {
      progress.progressMessage.destroy()
    })
    progresses.value.clear()
    return
  }
}

function handleButtonClick() {
  // 清理可能存在的舊計時器
  if (countdownInterval.value) {
    clearInterval(countdownInterval.value)
  }
  rejectCooldown.value = 10

  countdownInterval.value = setInterval(() => {
    rejectCooldown.value -= 1
    if (rejectCooldown.value <= 0) {
      clearInterval(countdownInterval.value)
    }
  }, 1000)
}
</script>

<template>
  <n-popconfirm :positive-text="null" :negative-text="null" v-model:show="popConfirmShowing">
    <div class="flex flex-col">
      <div>更新庫存是個大任務</div>
      <div>為了減輕禁漫伺服器壓力</div>
      <div>將自動調整配置中的下載間隔</div>
      <div>
        <span>之后你隨時可以在右上角的</span>
        <span class="bg-gray-2 px-1">配置</span>
        <span>調整</span>
      </div>
    </div>

    <template #action>
      <n-button size="small" :disabled="rejectButtonDisabled" @click="reject">
        <span v-if="rejectButtonDisabled">不調整直接下載 ({{ rejectCooldown }})</span>
        <span v-else>不調整直接下載</span>
      </n-button>
      <n-button size="small" type="primary" @click="agree">調整并下載</n-button>
    </template>

    <template #trigger>
      <n-button size="small" @click="handleButtonClick">更新庫存</n-button>
    </template>
  </n-popconfirm>
</template>
