import { api } from './client'
import type { Group, GroupConfig, ImportResult } from '../types/api'

export function fetchGroups() {
  return api.get<Group[]>('/groups')
}

export function createGroup(input: { name: string; medium: 'discord' | 'slack' }) {
  return api.post<Group>('/groups', input)
}

export function updateGroup(id: string, input: { name: string; medium: 'discord' | 'slack' }) {
  return api.put<Group>(`/groups/${id}`, input)
}

export function deleteGroup(id: string) {
  return api.del<null>(`/groups/${id}`)
}

export function pauseGroup(id: string) {
  return api.patch<Group>(`/groups/${id}/pause`)
}

export function resumeGroup(id: string) {
  return api.patch<Group>(`/groups/${id}/resume`)
}

export function fetchGroupConfig(id: string) {
  return api.get<GroupConfig>(`/groups/${id}/config`)
}

export function updateGroupConfig(id: string, config: GroupConfig) {
  return api.put<null>(`/groups/${id}/config`, config)
}

export function testSendGroup(
  id: string,
  body: Partial<{
    ticker: string
    companyName: string
    title: string
    evaluation: string
    embedColor: string
    webhookUrl: string
    mentionTargets: string[]
  }>,
) {
  return api.post<{ success: boolean; failureReason: string | null }>(
    `/groups/${id}/config/test-send`,
    body,
  )
}

export function bulkUpdateDestination(groupIds: string[], config: GroupConfig) {
  return api.put<{ updatedCount: number }>('/groups/bulk-destination', { groupIds, config })
}

export function importFiltersGlobal(rows: unknown[], dryRun: boolean) {
  return api.post<ImportResult>('/filters/import', { rows, dryRun })
}

export function importFiltersForGroup(groupId: string, rows: unknown[], dryRun: boolean) {
  return api.post<ImportResult>(`/groups/${groupId}/filters/import`, { rows, dryRun })
}
