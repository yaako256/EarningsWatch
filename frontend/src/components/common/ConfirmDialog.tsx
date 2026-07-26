// 削除・無効化等、破壊的操作の確認ダイアログ。
// 設計書9.4節: グループ削除は「delete」という固定文字列の入力を要求し、一致するまでボタンを無効化する。
// 単純な確認(はい/いいえ)で足りる操作(ユーザ無効化等)は requireTypedConfirmation を使わない。

import { useState } from 'react'
import { Modal } from './Modal'

interface ConfirmDialogProps {
  title: string
  description: string
  confirmLabel?: string
  requireTypedConfirmation?: string
  danger?: boolean
  isSubmitting?: boolean
  onConfirm: () => void
  onClose: () => void
}

export function ConfirmDialog({
  title,
  description,
  confirmLabel = '実行する',
  requireTypedConfirmation,
  danger = true,
  isSubmitting,
  onConfirm,
  onClose,
}: ConfirmDialogProps) {
  const [typed, setTyped] = useState('')
  const canConfirm = !requireTypedConfirmation || typed === requireTypedConfirmation

  return (
    <Modal title={title} onClose={onClose}>
      <p style={{ whiteSpace: 'pre-line', color: 'var(--text-muted)' }}>{description}</p>
      {requireTypedConfirmation && (
        <label>
          確認のため「{requireTypedConfirmation}」と入力してください
          <input value={typed} onChange={(e) => setTyped(e.target.value)} autoFocus />
        </label>
      )}
      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>キャンセル</button>
        <button
          className={danger ? 'danger-button' : 'primary'}
          disabled={!canConfirm || isSubmitting}
          onClick={onConfirm}
        >
          {isSubmitting ? '実行中...' : confirmLabel}
        </button>
      </div>
    </Modal>
  )
}
