// ユーザ管理画面。設計書9.10節 + 改善点まとめ資料 対応。
// 改善点資料: 仮ユーザ作成後、temporaryPasswordが表示されていなかった問題を修正する。
//
// Ver4改善点対応: バックエンドは未実装だが、以下をフロントエンド側だけ先行実装しておく
// (エンドポイント名は仮決め。バックエンド実装時に api/admin.ts のパスのみ調整すればよい)。
//   - ユーザーの有効化(無効化の対称操作)
//   - ユーザーの完全削除(取り消し不可のため、削除と同じ「delete」入力必須の確認ダイアログにする)
//   - パスワード再設定(仮ユーザ作成時と同様、temporaryPasswordを1回だけ表示する)

import { useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useAsync } from '../../hooks/useAsync'
import {
  fetchAdminUsers,
  createAdminUser,
  disableAdminUser,
  enableAdminUser,
  deleteAdminUserPermanently,
  resetAdminUserPassword,
  fetchAdminUserSummary,
} from '../../api/admin'
import { formatDateTime } from '../../utils/format'
import { copyToClipboard } from '../../utils/clipboard'
import { AtCoderPager } from '../../components/common/Pager'
import { EmptyState, LoadingRow, ErrorState } from '../../components/common/States'
import { Modal } from '../../components/common/Modal'
import { ConfirmDialog } from '../../components/common/ConfirmDialog'
import { useToast } from '../../contexts/ToastContext'
import { ApiError } from '../../api/client'
import type { AdminUser, AdminUserSummary } from '../../types/api'

export function AdminUsersTab() {
  const [searchParams, setSearchParams] = useSearchParams()
  const page = Number(searchParams.get('page') ?? '1')
  const { showToast } = useToast()

  const { data, isLoading, error, reload } = useAsync(() => fetchAdminUsers(page), [page])

  const [showCreate, setShowCreate] = useState(false)
  const [temporaryPasswordDisplay, setTemporaryPasswordDisplay] = useState<
    { username: string; password: string } | undefined
  >(undefined)
  const [summaryTarget, setSummaryTarget] = useState<AdminUser | undefined>(undefined)
  const [disableTarget, setDisableTarget] = useState<AdminUser | undefined>(undefined)
  const [enableTarget, setEnableTarget] = useState<AdminUser | undefined>(undefined)
  const [deleteTarget, setDeleteTarget] = useState<AdminUser | undefined>(undefined)
  const [resetPasswordTarget, setResetPasswordTarget] = useState<AdminUser | undefined>(undefined)
  const [busyId, setBusyId] = useState<string | undefined>(undefined)
  const passwordRef = useRef<HTMLElement>(null)

  const copyPassword = async (password: string) => {
    try {
      await copyToClipboard(password)
      showToast('success', 'コピーしました。')
    } catch {
      // クリップボードAPIが使えない環境向けに、テキストを選択状態にして手動コピーを促す。
      const selection = window.getSelection()
      const range = document.createRange()
      if (passwordRef.current) {
        range.selectNodeContents(passwordRef.current)
        selection?.removeAllRanges()
        selection?.addRange(range)
      }
      showToast(
        'error',
        'コピーに自動で失敗しました。パスワードの文字列を選択状態にしたので、Ctrl+C(Macは⌘+C)でコピーしてください。',
      )
    }
  }

  const handleDisable = async () => {
    if (!disableTarget) return
    setBusyId(disableTarget.id)
    try {
      await disableAdminUser(disableTarget.id)
      showToast('success', '無効化しました。')
      setDisableTarget(undefined)
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '操作に失敗しました。')
    } finally {
      setBusyId(undefined)
    }
  }

  const handleEnable = async () => {
    if (!enableTarget) return
    setBusyId(enableTarget.id)
    try {
      await enableAdminUser(enableTarget.id)
      showToast('success', '有効化しました。')
      setEnableTarget(undefined)
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
      await deleteAdminUserPermanently(deleteTarget.id)
      showToast('success', '完全に削除しました。')
      setDeleteTarget(undefined)
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '削除に失敗しました。')
    } finally {
      setBusyId(undefined)
    }
  }

  const handleResetPassword = async () => {
    if (!resetPasswordTarget) return
    setBusyId(resetPasswordTarget.id)
    try {
      const result = await resetAdminUserPassword(resetPasswordTarget.id)
      setTemporaryPasswordDisplay({ username: resetPasswordTarget.username, password: result.temporaryPassword })
      setResetPasswordTarget(undefined)
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : 'パスワード再設定に失敗しました。')
    } finally {
      setBusyId(undefined)
    }
  }

  return (
    <div>
      <div className="page-header-actions" style={{ marginBottom: 16 }}>
        <button className="primary" onClick={() => setShowCreate(true)}>
          + ユーザーを作成
        </button>
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <table>
          <thead>
            <tr>
              <th>ユーザー名</th>
              <th>ロール</th>
              <th>作成日</th>
              <th>状態</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={5} />}
            {!isLoading && data?.items.length === 0 && (
              <tr>
                <td colSpan={5}>
                  <EmptyState message="ユーザーがいません" />
                </td>
              </tr>
            )}
            {data?.items.map((u) => (
              <tr key={u.id}>
                <td>
                  <button className="link" onClick={() => setSummaryTarget(u)}>
                    {u.username}
                  </button>
                </td>
                <td>{u.role === 'admin' ? '管理者' : '一般'}</td>
                <td>{formatDateTime(u.createdAt)}</td>
                <td>
                  {u.disabledAt ? (
                    <span className="badge">無効</span>
                  ) : (
                    <span className="badge good">有効</span>
                  )}
                </td>
                <td>
                  <div className="cell-actions">
                    {u.disabledAt ? (
                      <button disabled={busyId === u.id} onClick={() => setEnableTarget(u)}>
                        有効化
                      </button>
                    ) : (
                      <button disabled={busyId === u.id} onClick={() => setDisableTarget(u)}>
                        無効化
                      </button>
                    )}
                    <button disabled={busyId === u.id} onClick={() => setResetPasswordTarget(u)}>
                      パスワード再設定
                    </button>
                    <button
                      className="danger-button"
                      disabled={busyId === u.id}
                      onClick={() => setDeleteTarget(u)}
                    >
                      完全削除
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {data && data.totalPages > 1 && (
        <AtCoderPager
          currentPage={page}
          totalPages={data.totalPages}
          onChange={(p) => setSearchParams({ page: String(p) })}
        />
      )}

      {showCreate && (
        <CreateUserModal
          onClose={() => setShowCreate(false)}
          onCreated={(username, password) => {
            setShowCreate(false)
            setTemporaryPasswordDisplay({ username, password })
            reload()
          }}
        />
      )}

      {temporaryPasswordDisplay && (
        <Modal title="仮パスワード" onClose={() => setTemporaryPasswordDisplay(undefined)}>
          <p className="password-warning">
            この仮パスワードはこの画面を閉じると二度と表示できません。必ずこの場でコピーし、
            利用者に安全な方法で伝達してください。
          </p>
          <p>
            ユーザー名: <strong>{temporaryPasswordDisplay.username}</strong>
          </p>
          <code className="password-display" ref={passwordRef}>
            {temporaryPasswordDisplay.password}
          </code>
          <button onClick={() => void copyPassword(temporaryPasswordDisplay.password)}>📋 コピー</button>
        </Modal>
      )}

      {summaryTarget && (
        <UserSummaryModal user={summaryTarget} onClose={() => setSummaryTarget(undefined)} />
      )}

      {disableTarget && (
        <ConfirmDialog
          title="ユーザーの無効化"
          description={`「${disableTarget.username}」を無効化します。ログインできなくなります。`}
          confirmLabel="無効化する"
          isSubmitting={busyId === disableTarget.id}
          onConfirm={handleDisable}
          onClose={() => setDisableTarget(undefined)}
        />
      )}

      {enableTarget && (
        <ConfirmDialog
          title="ユーザーの有効化"
          description={`「${enableTarget.username}」を有効化します。再びログインできるようになります。`}
          confirmLabel="有効化する"
          danger={false}
          isSubmitting={busyId === enableTarget.id}
          onConfirm={handleEnable}
          onClose={() => setEnableTarget(undefined)}
        />
      )}

      {deleteTarget && (
        <ConfirmDialog
          title="ユーザーの完全削除"
          description={`「${deleteTarget.username}」を完全に削除します。\nこのユーザーが持つグループ・フィルタ等もすべて連鎖して削除され、元に戻せません。`}
          requireTypedConfirmation="delete"
          confirmLabel="完全に削除する"
          isSubmitting={busyId === deleteTarget.id}
          onConfirm={handleDelete}
          onClose={() => setDeleteTarget(undefined)}
        />
      )}

      {resetPasswordTarget && (
        <ConfirmDialog
          title="パスワードの再設定"
          description={`「${resetPasswordTarget.username}」の新しい仮パスワードを発行します。現在のパスワードは無効になります。`}
          confirmLabel="再設定する"
          danger={false}
          isSubmitting={busyId === resetPasswordTarget.id}
          onConfirm={handleResetPassword}
          onClose={() => setResetPasswordTarget(undefined)}
        />
      )}
    </div>
  )
}

function CreateUserModal({
  onClose,
  onCreated,
}: {
  onClose: () => void
  onCreated: (username: string, password: string) => void
}) {
  const { showToast } = useToast()
  const [username, setUsername] = useState('')
  const [error, setError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async () => {
    setError('')
    if (username.trim() === '') {
      setError('ユーザー名を入力してください。')
      return
    }
    setIsSubmitting(true)
    try {
      const result = await createAdminUser(username)
      onCreated(result.username, result.temporaryPassword)
    } catch (err) {
      if (err instanceof ApiError && (err.code === 'invalid_request' || err.code === 'already_exists')) {
        setError(err.message)
      } else {
        showToast('error', err instanceof ApiError ? err.message : '作成に失敗しました。')
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Modal title="ユーザーを作成" onClose={onClose}>
      <label>
        ユーザー名<span className="required-mark">*</span>
        <input value={username} onChange={(e) => setUsername(e.target.value)} autoFocus />
      </label>
      {error && <p className="inline-error">{error}</p>}
      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>キャンセル</button>
        <button className="primary" disabled={isSubmitting} onClick={handleSubmit}>
          {isSubmitting ? '作成中...' : '作成する'}
        </button>
      </div>
    </Modal>
  )
}

function UserSummaryModal({ user, onClose }: { user: AdminUser; onClose: () => void }) {
  const { data, isLoading, error } = useAsync(() => fetchAdminUserSummary(user.id), [user.id])

  return (
    <Modal title={`${user.username} の利用状況`} onClose={onClose}>
      {isLoading && (
        <p className="loading">
          <i /> 読み込み中...
        </p>
      )}
      {error && <ErrorState message={error} />}
      {data && <SummaryCards data={data} />}
    </Modal>
  )
}

function SummaryCards({ data }: { data: AdminUserSummary }) {
  return (
    <div className="cards small-cards">
      <div className="card">
        <small>グループ数</small>
        <strong>{data.groupCount}</strong>
      </div>
      <div className="card">
        <small>フィルタ数</small>
        <strong>{data.filterCount}</strong>
      </div>
      <div className="card">
        <small>Discordグループ</small>
        <strong>{data.discordGroupCount}</strong>
      </div>
      <div className="card">
        <small>Slackグループ</small>
        <strong>{data.slackGroupCount}</strong>
      </div>
    </div>
  )
}
