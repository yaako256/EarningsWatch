// グループ詳細 - 設定タブ。設計書9.5節対応。
// テスト送信モーダルは「シンプル版」「カスタム版」のタブ切り替え。

import { useEffect, useState } from 'react'
import { useOutletContext } from 'react-router-dom'
import { GroupConfigForm } from '../../components/common/GroupConfigForm'
import { Modal } from '../../components/common/Modal'
import { useAsync } from '../../hooks/useAsync'
import { fetchGroupConfig, updateGroupConfig, testSendGroup } from '../../api/groups'
import { useToast } from '../../contexts/ToastContext'
import { ApiError } from '../../api/client'
import type { Group, GroupConfig } from '../../types/api'
import { ErrorState } from '../../components/common/States'

interface OutletContext {
  group: Group
}

export function GroupSettingsTab() {
  const { group } = useOutletContext<OutletContext>()
  const { showToast } = useToast()
  const { data, isLoading, error, reload } = useAsync(() => fetchGroupConfig(group.id), [group.id])

  const [draft, setDraft] = useState<GroupConfig | undefined>(undefined)
  const [isSaving, setIsSaving] = useState(false)
  const [showTestSend, setShowTestSend] = useState(false)

  useEffect(() => {
    if (data) setDraft(data)
  }, [data])

  const handleSave = async () => {
    if (!draft) return
    setIsSaving(true)
    try {
      await updateGroupConfig(group.id, draft)
      showToast('success', '設定を保存しました。')
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '保存に失敗しました。')
    } finally {
      setIsSaving(false)
    }
  }

  if (isLoading) {
    return (
      <p className="loading">
        <i /> 読み込み中...
      </p>
    )
  }
  if (error) return <ErrorState message={error} onRetry={reload} />
  if (!draft) return null

  return (
    <div>
      <GroupConfigForm config={draft} onChange={setDraft} />

      <div className="page-header-actions" style={{ marginTop: 24 }}>
        <button onClick={() => setShowTestSend(true)}>🧪 テスト送信</button>
        <button className="primary" disabled={isSaving} onClick={handleSave}>
          {isSaving ? '保存中...' : '設定を保存'}
        </button>
      </div>

      {showTestSend && (
        <TestSendModal
          groupId={group.id}
          medium={group.medium}
          savedConfig={data}
          onClose={() => setShowTestSend(false)}
        />
      )}
    </div>
  )
}

function TestSendModal({
  groupId,
  medium,
  savedConfig,
  onClose,
}: {
  groupId: string
  medium: string
  savedConfig?: GroupConfig
  onClose: () => void
}) {
  const { showToast } = useToast()
  const [mode, setMode] = useState<'simple' | 'custom'>('simple')
  const [isSending, setIsSending] = useState(false)
  const [custom, setCustom] = useState({
    ticker: '',
    companyName: '',
    title: '',
    evaluation: 'unrated',
  })

  const send = async () => {
    setIsSending(true)
    try {
      const body = mode === 'simple' ? {} : custom
      const result = await testSendGroup(groupId, body)
      if (result.success) {
        showToast('success', 'テスト送信に成功しました。')
      } else {
        showToast('error', result.failureReason ?? 'テスト送信に失敗しました。')
      }
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : 'テスト送信に失敗しました。')
    } finally {
      setIsSending(false)
    }
  }

  return (
    <Modal title="テスト送信" onClose={onClose}>
      <div className="tabs">
        <button className={mode === 'simple' ? 'active' : ''} onClick={() => setMode('simple')}>
          シンプル版
        </button>
        <button className={mode === 'custom' ? 'active' : ''} onClick={() => setMode('custom')}>
          カスタム版
        </button>
      </div>

      {mode === 'simple' && (
        <p style={{ color: 'var(--text-muted)' }}>
          保存済みの設定({medium === 'discord' ? 'Discord' : 'Slack'} / Webhook:{' '}
          {savedConfig?.webhookUrl ? '設定済み' : '未設定'})をそのまま使ってテスト送信します。
        </p>
      )}

      {mode === 'custom' && (
        <div className="form-grid">
          <label>
            証券コード
            <input
              placeholder="1234"
              value={custom.ticker}
              onChange={(e) => setCustom({ ...custom, ticker: e.target.value })}
            />
          </label>
          <label>
            銘柄名
            <input
              placeholder="サンプル株式会社"
              value={custom.companyName}
              onChange={(e) => setCustom({ ...custom, companyName: e.target.value })}
            />
          </label>
          <label style={{ gridColumn: '1 / -1' }}>
            タイトル
            <input
              placeholder="2026年3月期 決算発表"
              value={custom.title}
              onChange={(e) => setCustom({ ...custom, title: e.target.value })}
            />
          </label>
          <label>
            評価
            <select
              value={custom.evaluation}
              onChange={(e) => setCustom({ ...custom, evaluation: e.target.value })}
            >
              <option value="unrated">未評価</option>
              <option value="positive">ポジティブ</option>
              <option value="neutral">ニュートラル</option>
              <option value="negative">ネガティブ</option>
            </select>
          </label>
        </div>
      )}

      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>閉じる</button>
        <button className="primary" disabled={isSending} onClick={send}>
          {isSending ? '送信中...' : '送信する'}
        </button>
      </div>
    </Modal>
  )
}
