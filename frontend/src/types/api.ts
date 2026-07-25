// API設計書に基づく型定義。
// フィールド名・enum値は API設計書.md の記述にそのまま準拠する(camelCase)。

export type Role = 'admin' | 'user'
export type NotifyMedium = 'discord' | 'slack'
export type PageType = 'blog' | 'static'
export type EarningsEvaluation = 'positive' | 'neutral' | 'negative' | 'unrated'
export type NotifyStatus = 'ready' | 'sent' | 'failed'
export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error'
export type LogProcess = 'server' | 'monitor' | 'notify'
export type SystemRunType = 'monitor' | 'notify'

export interface User {
  username: string
  role: Role
}

export interface Page<T> {
  items: T[]
  page: number
  perPage: number
  totalCount: number
  totalPages: number
}

export interface Group {
  id: string
  name: string
  medium: NotifyMedium
  pausedAt: string | null
  createdAt: string
  updatedAt: string
}

export interface DiscordGroupConfig {
  medium: 'discord'
  webhookUrl: string | null
  embedColor: string | null
  mentionEnabled: boolean
  mentionTargets: string[]
}

export interface SlackGroupConfig {
  medium: 'slack'
  webhookUrl: string | null
  mentionEnabled: boolean
  mentionTargets: string[]
}

export type GroupConfig = DiscordGroupConfig | SlackGroupConfig

export interface Filter {
  id: string
  groupId: string
  ticker: string
  companyName: string
  notes: string | null
  enabled: boolean
}

export interface ImportRow {
  ticker: string
  companyName: string
  groupName?: string
  notes?: string | null
  enabled?: boolean | null
}

export interface ImportResult {
  importedCount: number
  skippedEmptyRows: number
  duplicateCount: number
  errorRows: { rowNumber: number; reason: string }[]
  createdGroups: { id: string; name: string }[]
  pausedGroups: { id: string; name: string }[]
  warnings: { rowNumber: number; message: string }[]
}

export interface Earnings {
  id: number
  ticker: string
  companyName: string
  publishedAt: string
  title: string
  url: string
  summary: string
  evaluation: EarningsEvaluation
}

export interface DailyCount {
  dateJst: string
  count: number
}

export interface NotifyQueueItem {
  id: number
  ticker: string
  companyName: string
  publishedAt: string
  title: string
  evaluation: EarningsEvaluation
  status: NotifyStatus
}

export interface NotifyHistoryItem {
  id: number
  groupId: string | null
  groupName: string | null
  fingerprint: string
  sentAt: string
  status: NotifyStatus
}

export interface Dashboard {
  groupCount: number
  filterCount: number
  uniqueTickerCount: number
  uniqueCompanyNameCount: number
  mediumBreakdown: { discord: number; slack: number }
  pausedGroupCount: number
  webhookMissingCount: number
  recentSent: NotifyHistoryItem[]
  recentFailed: NotifyHistoryItem[]
}

export interface PageListItem {
  id: string
  title: string
  isPublished: boolean
  createdAt: string
  updatedAt: string
  displayOrder: number | null
  authorUsername: string
}

export interface PageDetail {
  id: string
  type: PageType
  title: string
  contentMarkdown: string
  displayOrder: number | null
  isPublished: boolean
  createdAt: string
  updatedAt: string
  authorUsername: string
}

export interface LogEntry {
  id: number
  timestamp: string
  level: LogLevel
  process: LogProcess
  target: string
  message: string | null
  fields: Record<string, unknown>
}

export interface AdminUser {
  id: string
  username: string
  role: Role
  createdAt: string
  disabledAt: string | null
}

export interface AdminUserSummary {
  groupCount: number
  filterCount: number
  discordGroupCount: number
  slackGroupCount: number
}

export interface RunDuration {
  runType: SystemRunType
  runAt: string
  durationMs: number
}

export interface AdminDashboard {
  totalEarningsCount: number
  notifySuccessRate: number | null
  lastMonitorRunAt: string | null
  runDurations: RunDuration[]
}

export type ErrorCode =
  | 'unauthorized'
  | 'forbidden'
  | 'not_found'
  | 'already_exists'
  | 'invalid_request'
  | 'notify_config_missing'
  | 'notify_send_failed'
  | 'notify_rejected'
  | 'import_empty'
  | 'internal_error'
