// 共通APIラッパー。
//
// - すべてのリクエストに credentials: "include" を付与する(Cookie認証のため)。
// - 401 (unauthorized) を検知した場合、/auth/refresh を1回だけ試みて元のリクエストを再試行する。
//   再試行してもなお失敗する場合は onUnauthorized を呼び、呼び出し側(AuthContext)でログイン画面に戻す。
// - レスポンスは常に { data, error } のエンベロープ形式(API設計書 2.1節)。

import type { ErrorCode, Role } from '../types/api'
import { errorCodeMessage } from '../utils/errorMessages'

export class ApiError extends Error {
  code: ErrorCode | string
  status: number

  constructor(code: string, message: string, status: number) {
    super(message)
    this.code = code
    this.status = status
  }
}

interface Envelope<T> {
  data: T | null
  error: { code: string; message: string } | null
}

// refresh のリクエストが複数同時に走らないよう、進行中の Promise を使い回す。
let refreshing: Promise<boolean> | undefined

async function refreshAccessToken(): Promise<boolean> {
  refreshing ??= fetch('/api/auth/refresh', { method: 'POST', credentials: 'include' })
    .then(async (res) => {
      if (!res.ok) return false
      const body = (await res.json()) as Envelope<null>
      return body.error === null
    })
    .catch(() => false)
    .finally(() => {
      refreshing = undefined
    })
  return refreshing
}

class ApiClient {
  /** 認証切れ(refreshも失敗)を検知した際に呼ばれるコールバック。AuthContext がセットする。 */
  onUnauthorized: (() => void) | undefined

  async request<T>(path: string, init: RequestInit = {}, isRetry = false): Promise<T> {
    const response = await fetch('/api' + path, {
      ...init,
      credentials: 'include',
      headers: {
        ...(init.body ? { 'Content-Type': 'application/json' } : {}),
        ...init.headers,
      },
    })
    const body = (await response.json()) as Envelope<T>

    const isUnauthorized = response.status === 401
    const canRetry = !isRetry && path !== '/auth/refresh'
    if (isUnauthorized && canRetry) {
      const refreshed = await refreshAccessToken()
      if (refreshed) {
        return this.request<T>(path, init, true)
      }
      this.onUnauthorized?.()
    }

    if (!response.ok || body.error) {
      const code = body.error?.code ?? 'internal_error'
      throw new ApiError(code, errorCodeMessage(code), response.status)
    }
    return body.data as T
  }

  get<T>(path: string) {
    return this.request<T>(path)
  }

  post<T>(path: string, body?: unknown) {
    return this.request<T>(path, {
      method: 'POST',
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  }

  put<T>(path: string, body: unknown) {
    return this.request<T>(path, { method: 'PUT', body: JSON.stringify(body) })
  }

  patch<T>(path: string, body?: unknown) {
    return this.request<T>(path, {
      method: 'PATCH',
      body: body === undefined ? undefined : JSON.stringify(body),
    })
  }

  del<T>(path: string) {
    return this.request<T>(path, { method: 'DELETE' })
  }

  login(username: string, password: string) {
    return this.post<{ username: string; role: Role }>('/auth/login', { username, password })
  }
}

export const api = new ApiClient()

/** クエリパラメータをsnake_caseのまま組み立てる。undefined/null/空文字は除外する。 */
export function qs(params: Record<string, unknown>): string {
  const entries = Object.entries(params).filter(
    ([, value]) => value !== undefined && value !== null && value !== '',
  )
  return new URLSearchParams(entries.map(([key, value]) => [key, String(value)])).toString()
}

/**
 * xlsx等のバイナリファイルをダウンロードする。
 * name にはユニーク性を持たせるため、呼び出し側で日時プレフィックスを付与すること(utils/exportFileName.ts参照)。
 */
export async function downloadFile(path: string, filename: string): Promise<void> {
  const response = await fetch('/api' + path, { credentials: 'include' })
  if (!response.ok) {
    throw new Error('ダウンロードに失敗しました。')
  }
  const blob = await response.blob()
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}
