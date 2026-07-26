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

// Ver4改善点対応: バックエンド未実装だが、無効化(disable)と対称になる形で
// フロントエンド側だけ先行実装しておく。エンドポイント名は仮決め。
// バックエンド実装時にパスが変わる場合はここだけ修正すればよい。
export function enableAdminUser(id: string) {
  return api.post<null>(`/admin/users/${id}/enable`)
}

// 完全削除(disable/enableと違い元に戻せない操作のため、確認ダイアログでの
// タイプ確認必須とセットで運用する想定)。エンドポイント名・メソッドは仮決め。
export function deleteAdminUserPermanently(id: string) {
  return api.del<null>(`/admin/users/${id}`)
}

// パスワード再設定(仮ユーザ作成時と同様、temporaryPasswordを1回だけ返す想定)。
// エンドポイント名は仮決め。
export function resetAdminUserPassword(id: string) {
  return api.post<{ temporaryPassword: string }>(`/admin/users/${id}/reset-password`)
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
