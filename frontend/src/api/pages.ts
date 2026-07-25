import { api, qs } from './client'
import type { PageDetail, PageListItem, PageType } from '../types/api'

export function fetchPageList(type: PageType) {
  return api.get<PageListItem[]>(`/pages?${qs({ type })}`)
}

export function fetchPageDetail(id: string) {
  return api.get<PageDetail>(`/pages/${id}`)
}

export function createPage(input: {
  type: PageType
  title: string
  contentMarkdown: string
  displayOrder: number | null
  isPublished: boolean
}) {
  return api.post<PageDetail>('/pages', input)
}

export function updatePage(
  id: string,
  input: { title: string; contentMarkdown: string; isPublished: boolean },
) {
  return api.put<PageDetail>(`/pages/${id}`, input)
}

export function deletePage(id: string) {
  return api.del<null>(`/pages/${id}`)
}

export function reorderPage(id: string, displayOrder: number) {
  return api.patch<PageDetail>(`/pages/${id}/order`, { displayOrder })
}
