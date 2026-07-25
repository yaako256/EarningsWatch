// システム通知設定画面。設計書9.11節対応。GroupConfigFormを共有する。テスト送信は付けない(現行API未対応のため)。

import { useEffect, useState } from 'react'
import { GroupConfigForm } from '../../components/common/GroupConfigForm'
import { useAsync } from '../../hooks/useAsync'
import { fetchSystemNotifyConfig, updateSystemNotifyConfig } from '../../api/admin'
import { useToast } from '../../contexts/ToastContext'
import { ApiError } from '../../api/client'
import { ErrorState } from '../../components/common/States'
import type { GroupConfig } from '../../types/api'

const DEFAULT_CONFIG: GroupConfig = {
  medium: 'discord',
  webhookUrl: null,
  embedColor: null,
  mentionEnabled: false,
  mentionTargets: [],
}

export function AdminNotifyConfigTab() {
  const { showToast } = useToast()
  const { data, isLoading, error, reload } = useAsync(fetchSystemNotifyConfig, [])
  const [draft, setDraft] = useState<GroupConfig>(DEFAULT_CONFIG)
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    if (data) setDraft(data)
  }, [data])

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await updateSystemNotifyConfig(draft)
      showToast('success', '保存しました。')
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

  return (
    <div>
      <p style={{ color: 'var(--text-muted)' }}>
        システム全体の警告・異常通知の送信先を設定します。
      </p>
      <label>
        通知媒体
        <select
          value={draft.medium}
          onChange={(e) =>
            setDraft(
              e.target.value === 'discord'
                ? { ...draft, medium: 'discord', embedColor: draft.medium === 'discord' ? draft.embedColor : null }
                : { medium: 'slack', webhookUrl: draft.webhookUrl, mentionEnabled: draft.mentionEnabled, mentionTargets: draft.mentionTargets },
            )
          }
        >
          <option value="discord">Discord</option>
          <option value="slack">Slack(開発中)</option>
        </select>
      </label>
      <GroupConfigForm config={draft} onChange={setDraft} />
      <button className="primary" disabled={isSaving} onClick={handleSave}>
        {isSaving ? '保存中...' : '設定を保存'}
      </button>
    </div>
  )
}
