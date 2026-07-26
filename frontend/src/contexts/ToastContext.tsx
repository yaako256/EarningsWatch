// トースト通知の共通管理。
// 設計書 11.6節: 右下表示・5秒で自動消滅・閉じるボタン・成功緑/エラー赤の色帯・新しい通知は下に追加。

import { createContext, useCallback, useContext, useState } from 'react'
import type { ReactNode } from 'react'

export type ToastKind = 'success' | 'error'

interface Toast {
  id: number
  kind: ToastKind
  text: string
}

interface ToastContextValue {
  toasts: Toast[]
  showToast: (kind: ToastKind, text: string) => void
  dismissToast: (id: number) => void
}

const ToastContext = createContext<ToastContextValue | undefined>(undefined)

const AUTO_DISMISS_MS = 5000

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([])

  const dismissToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id))
  }, [])

  const showToast = useCallback(
    (kind: ToastKind, text: string) => {
      const id = Date.now() + Math.random()
      setToasts((prev) => [...prev, { id, kind, text }])
      setTimeout(() => dismissToast(id), AUTO_DISMISS_MS)
    },
    [dismissToast],
  )

  return (
    <ToastContext.Provider value={{ toasts, showToast, dismissToast }}>
      {children}
    </ToastContext.Provider>
  )
}

export function useToast(): ToastContextValue {
  const context = useContext(ToastContext)
  if (!context) throw new Error('useToast must be used within ToastProvider')
  return context
}
