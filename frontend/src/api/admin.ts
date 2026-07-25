import { api, qs } from './client'
import type {
  AdminDashboard,
  AdminUser,
  AdminUserSummary,
  GroupConfig,
  LogEntry,
  Page,
} from '../types/api'

export function fetchAdminUsers(page: number, perPage = 20) {
  return api.get<Page<AdminUser>>(`/admin/users?${qs({ page, per_page: perPage })}`)
}

export function createAdminUser(username: string) {
  return api.post<{ id: string; username: string; temporaryPassword: string }>('/admin/users', {
    username,
  })
}

export function disableAdminUser(id: string) {
  return api.post<null>(`/admin/users/${id}/disable`)
}

export function fetchAdminUserSummary(id: string) {
  return api.get<AdminUserSummary>(`/admin/users/${id}/summary`)
}

export function fetchSystemNotifyConfig() {
  return api.get<GroupConfig | null>('/admin/notify-config')
}

export function updateSystemNotifyConfig(config: GroupConfig) {
  return api.put<null>('/admin/notify-config', config)
}

export function fetchAdminDashboard() {
  return api.get<AdminDashboard>('/admin/dashboard')
}

export interface LogFilters {
  from?: string
  to?: string
  level?: string
  process?: string
}

export function fetchAdminLogs(filters: LogFilters, page: number, perPage = 30) {
  return api.get<Page<LogEntry>>(`/admin/logs?${qs({ ...filters, page, per_page: perPage })}`)
}
