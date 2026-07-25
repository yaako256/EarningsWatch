import { api, qs } from './client'
import type { DailyCount, Earnings, Page } from '../types/api'

export interface EarningsFilters {
  ticker?: string
  company_name?: string
  evaluation?: string
  from?: string
  to?: string
}

export function fetchEarnings(filters: EarningsFilters, page: number, perPage = 20) {
  return api.get<Page<Earnings>>(`/earnings?${qs({ ...filters, page, per_page: perPage })}`)
}

export function fetchEarningsSummary(from?: string, to?: string) {
  return api.get<{ dailyCounts: DailyCount[] }>(`/earnings/summary?${qs({ from, to })}`)
}
