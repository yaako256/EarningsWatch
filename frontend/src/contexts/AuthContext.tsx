// 認証状態を一元管理するContext。
// フロントエンド設計書 4章の決定事項に準拠:
//   - 起動時に /auth/me を呼び、結果をセットする
//   - 確認完了までは <AuthLoading /> を表示する
//   - 401検知時はApiClient.onUnauthorizedからlogout()相当の状態リセットが呼ばれる

import { createContext, useCallback, useContext, useEffect, useState } from 'react'
import type { ReactNode } from 'react'
import { api } from '../api/client'
import { fetchCurrentUser, login as apiLogin, logout as apiLogout } from '../api/auth'
import type { User } from '../types/api'

interface AuthContextValue {
  user: User | null
  isLoading: boolean
  login: (username: string, password: string) => Promise<void>
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    api.onUnauthorized = () => setUser(null)
    fetchCurrentUser()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setIsLoading(false))
  }, [])

  const login = useCallback(async (username: string, password: string) => {
    const result = await apiLogin(username, password)
    setUser(result)
  }, [])

  const logout = useCallback(async () => {
    await apiLogout().catch(() => {})
    setUser(null)
  }, [])

  return (
    <AuthContext.Provider value={{ user, isLoading, login, logout }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext)
  if (!context) throw new Error('useAuth must be used within AuthProvider')
  return context
}
