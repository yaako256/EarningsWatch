// グループ一覧ページ。設計書9.4節 + 改善点まとめ資料 対応。
//
// 改善点資料への対応:
//   - 一括送信先設定機能は一旦削除(バックエンド側の対応待ちのため)
//   - 一時停止中の時刻表記に「〜」を付け、「いつからの停止か」を明確にする
//   - 全体一括インポート機能を追加(以前は存在しなかった)

import { useState } from 'react'
import { Link } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAsync } from '../hooks/useAsync'
import { fetchGroups, createGroup, deleteGroup, pauseGroup, resumeGroup, importFiltersGlobal } from '../api/groups'
import { formatSince, exportFileName } from '../utils/format'
import { EmptyState, LoadingRow } from '../components/common/States'
import { ErrorState } from '../components/common/States'
import { Modal } from '../components/common/Modal'
import { ConfirmDialog } from '../components/common/ConfirmDialog'
import { ImportModal } from '../components/common/ImportModal'
import { useToast } from '../contexts/ToastContext'
import { ApiError, downloadFile, qs } from '../api/client'
import type { Group, ImportRow } from '../types/api'

const NAME_MAX_LENGTH = 30

export function GroupsPage() {
  const { data, isLoading, error, reload } = useAsync(fetchGroups, [])
  const { showToast } = useToast()

  const [showCreate, setShowCreate] = useState(false)
  const [showImport, setShowImport] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<Group | undefined>(undefined)
  const [busyId, setBusyId] = useState<string | undefined>(undefined)

  const togglePause = async (group: Group) => {
    setBusyId(group.id)
    try {
      if (group.pausedAt) {
        await resumeGroup(group.id)
        showToast('success', `${group.name} を再開しました。`)
      } else {
        await pauseGroup(group.id)
        showToast('success', `${group.name} を一時停止しました。`)
      }
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '操作に失敗しました。')
    } finally {
      setBusyId(undefined)
    }
  }

  const handleDelete = async () => {
    if (!deleteTarget) return
    setBusyId(deleteTarget.id)
    try {
      await deleteGroup(deleteTarget.id)
      showToast('success', `${deleteTarget.name} を削除しました。`)
      setDeleteTarget(undefined)
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '削除に失敗しました。')
    } finally {
      setBusyId(undefined)
    }
  }

  // API設計書7.3節: /filters/export は format クエリ(現状xlsxのみ)が必須。
  const handleExport = async () => {
    try {
      await downloadFile(`/filters/export?${qs({ format: 'xlsx' })}`, exportFileName('all_group_filters.xlsx'))
    } catch {
      showToast('error', 'エクスポートに失敗しました。')
    }
  }

  return (
    <div>
      <PageHeader
        icon="👥"
        title="グループ管理"
        actions={
          <>
            <button onClick={() => setShowImport(true)}>⇪ 一括インポート</button>
            <button onClick={handleExport}>⇩ 全フィルタをエクスポート</button>
            <button className="primary" onClick={() => setShowCreate(true)}>
              + グループを作成
            </button>
          </>
        }
      />

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <table>
          <thead>
            <tr>
              <th>グループ名</th>
              <th>媒体</th>
              <th>状態</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={4} />}
            {!isLoading && data?.length === 0 && (
              <tr>
                <td colSpan={4}>
                  <EmptyState message="グループがまだありません。「グループを作成」から追加してください" />
                </td>
              </tr>
            )}
            {data?.map((group) => (
              <tr key={group.id}>
                <td>
                  <Link className="link" to={`/groups/${group.id}`}>
                    {group.name}
                  </Link>
                </td>
                <td>
                  <span className="badge">{group.medium === 'discord' ? 'Discord' : 'Slack'}</span>
                </td>
                <td>
                  {group.pausedAt ? (
                    <span className="badge">一時停止中({formatSince(group.pausedAt)})</span>
                  ) : (
                    <span className="badge good">稼働中</span>
                  )}
                </td>
                <td>
                  <div className="cell-actions">
                    <button disabled={busyId === group.id} onClick={() => togglePause(group)}>
                      {group.pausedAt ? '再開' : '一時停止'}
                    </button>
                    <button
                      className="danger-button"
                      disabled={busyId === group.id}
                      onClick={() => setDeleteTarget(group)}
                    >
                      削除
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {showCreate && (
        <CreateGroupModal
          onClose={() => setShowCreate(false)}
          onCreated={() => {
            setShowCreate(false)
            reload()
          }}
        />
      )}

      {deleteTarget && (
        <ConfirmDialog
          title="グループの削除"
          description={`「${deleteTarget.name}」を削除します。\nこのグループに紐づくフィルタ・設定もすべて連鎖して削除され、元に戻せません。`}
          requireTypedConfirmation="delete"
          confirmLabel="削除する"
          isSubmitting={busyId === deleteTarget.id}
          onConfirm={handleDelete}
          onClose={() => setDeleteTarget(undefined)}
        />
      )}

      {showImport && (
        <ImportModal
          title="フィルタの一括インポート(全体)"
          scopeHint="groupNameが既存グループ名と一致しない場合、新しいグループが作成されます。"
          onImport={(rows: ImportRow[], dryRun) => importFiltersGlobal(rows, dryRun)}
          onClose={() => setShowImport(false)}
          onImported={reload}
        />
      )}
    </div>
  )
}

function CreateGroupModal({ onClose, onCreated }: { onClose: () => void; onCreated: () => void }) {
  const { showToast } = useToast()
  const [name, setName] = useState('')
  const [medium, setMedium] = useState<'discord' | 'slack'>('discord')
  const [nameError, setNameError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [showSlackWarning, setShowSlackWarning] = useState(false)

  const handleSubmit = async () => {
    setNameError('')
    if (name.length < 1 || name.length > NAME_MAX_LENGTH) {
      setNameError(`グループ名は1〜${NAME_MAX_LENGTH}文字で入力してください。`)
      return
    }
    setIsSubmitting(true)
    try {
      await createGroup({ name, medium })
      showToast('success', `${name} を作成しました。`)
      onCreated()
    } catch (err) {
      if (err instanceof ApiError && err.code === 'invalid_request') {
        setNameError(err.message)
      } else {
        showToast('error', err instanceof ApiError ? err.message : '作成に失敗しました。')
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Modal title="グループを作成" onClose={onClose}>
      <label>
        グループ名<span className="required-mark">*</span>
        <input value={name} onChange={(e) => setName(e.target.value)} maxLength={NAME_MAX_LENGTH} autoFocus />
      </label>
      {nameError && <p className="inline-error">{nameError}</p>}

      <label>
        通知媒体<span className="required-mark">*</span>
        <select
          value={medium}
          onChange={(e) => {
            const value = e.target.value as 'discord' | 'slack'
            setMedium(value)
            setShowSlackWarning(value === 'slack')
          }}
        >
          <option value="discord">Discord</option>
          <option value="slack">Slack(開発中)</option>
        </select>
      </label>
      {showSlackWarning && (
        <p className="inline-error">
          Slackは現時点で開発中の仮実装のため、実質的にご利用いただけません。
        </p>
      )}

      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>キャンセル</button>
        <button className="primary" disabled={isSubmitting} onClick={handleSubmit}>
          {isSubmitting ? '作成中...' : '作成する'}
        </button>
      </div>
    </Modal>
  )
}
