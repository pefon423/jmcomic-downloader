import { DownloadEvent } from './bindings.ts'

export type CurrentTabName = 'search' | 'favorite' | 'serial' | 'hanman' | 'downloaded' | 'chapter'
export type ProgressesPaneTabName = 'uncompleted' | 'completed' | 'export'

export type ProgressData = Extract<DownloadEvent, { event: 'TaskCreate' }>['data'] & {
  percentage: number
  indicator: string
}
