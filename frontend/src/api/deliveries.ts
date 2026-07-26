import { api, qs } from './client'
import type { Dashboard, NotifyHistoryItem, NotifyQueueItem, Page } from '../types/api'

export function fetchNotifyQueue(status: string | undefined, page: number, perPage = 20) {
  return api.get<Page<NotifyQueueItem>>(`/notify-queue?${qs({ status, page, per_page: perPage })}`)
}

export function fetchNotifyHistory(
  groupId: string | undefined,
  status: string | undefined,
  page: number,
  perPage = 20,
) {
  return api.get<Page<NotifyHistoryItem>>(
    `/notify-history?${qs({ group_id: groupId, status, page, per_page: perPage })}`,
  )
}

export function fetchDashboard() {
  return api.get<Dashboard>('/dashboard')
}
