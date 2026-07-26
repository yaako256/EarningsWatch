import { api, qs } from './client'
import type { Filter, Page } from '../types/api'

// 設計書 9.6節: サーバー側に検索パラメータが存在しないため、per_pageを大きな固定値にして
// 事実上全件を1回で取得し、検索・絞り込みはクライアントサイドで行う。
const FETCH_ALL_PER_PAGE = 1000

export function fetchGroupFilters(groupId: string) {
  return api.get<Page<Filter>>(`/groups/${groupId}/filters?${qs({ page: 1, per_page: FETCH_ALL_PER_PAGE })}`)
}

export function createFilter(
  groupId: string,
  input: { ticker: string; companyName: string; notes: string | null },
) {
  return api.post<Filter>(`/groups/${groupId}/filters`, input)
}

export function updateFilter(
  groupId: string,
  filterId: string,
  input: { ticker: string; companyName: string; notes: string | null },
) {
  return api.put<Filter>(`/groups/${groupId}/filters/${filterId}`, input)
}

export function setFilterEnabled(groupId: string, filterId: string, enabled: boolean) {
  return api.patch<null>(`/groups/${groupId}/filters/${filterId}/${enabled ? 'enable' : 'disable'}`)
}

export function deleteFilter(groupId: string, filterId: string) {
  return api.del<null>(`/groups/${groupId}/filters/${filterId}`)
}

export function bulkFilterAction(action: 'enable' | 'disable' | 'delete', filterIds: string[]) {
  return api.post<{ updatedCount: number }>(`/filters/bulk-${action}`, { filterIds })
}
