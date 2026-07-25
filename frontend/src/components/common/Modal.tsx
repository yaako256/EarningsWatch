// 共通モーダル。
// 設計書11.7節: 背景クリックでは閉じない。明示的な閉じるボタン(バツ)でのみ閉じる。

import type { ReactNode } from 'react'

interface ModalProps {
  title: string
  onClose: () => void
  children: ReactNode
  wide?: boolean
}

export function Modal({ title, onClose, children, wide }: ModalProps) {
  return (
    <div className="modal-backdrop">
      <div className="modal" style={wide ? { width: 'min(880px, calc(100vw - 40px))' } : undefined}>
        <header>
          <h2>{title}</h2>
          <button className="icon-button" onClick={onClose} aria-label="閉じる">
            ✕
          </button>
        </header>
        {children}
      </div>
    </div>
  )
}
