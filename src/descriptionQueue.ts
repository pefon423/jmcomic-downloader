import { commands } from './bindings.ts'

// 背景補抓漫畫描述的佇列：先查本地快取，沒有才發請求，並且每次請求之間有間隔，避免被限流
const REQUEST_GAP_MS = 1200

interface Job {
  comicId: number
  apply: (description: string) => void
  cancelled: () => boolean
}

const queue: Job[] = []
let running = false

export function enqueueDescription(job: Job) {
  queue.push(job)
  void run()
}

async function run() {
  if (running) {
    return
  }
  running = true
  try {
    while (queue.length > 0) {
      const job = queue.shift()
      if (job === undefined || job.cancelled()) {
        continue
      }

      const cached = await commands.getCachedDescription(job.comicId)
      if (cached !== null) {
        job.apply(cached)
        continue
      }

      const result = await commands.fetchComicDescription(job.comicId)
      if (result.status === 'ok') {
        job.apply(result.data)
      } else {
        console.error(result.error)
      }
      await new Promise((resolve) => setTimeout(resolve, REQUEST_GAP_MS))
    }
  } finally {
    running = false
  }
}
