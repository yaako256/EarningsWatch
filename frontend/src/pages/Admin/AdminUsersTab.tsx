// ユーザ管理画面。設計書9.10節 + 改善点まとめ資料 対応。
// 改善点資料: 仮ユーザ作成後、temporaryPasswordが表示されていなかった問題を修正する。

import { useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useAsync } from '../../hooks/useAsync'
import { fetchAdminUsers, createAdminUser, disableAdminUser, fetchAdminUserSummary } from '../../api/admin'
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
  const [createdPassword, setCreatedPassword] = useState<{ username: string; password: string } | undefined>(
    undefined,
  )
  const [summaryTarget, setSummaryTarget] = useState<AdminUser | undefined>(undefined)
  const [disableTarget, setDisableTarget] = useState<AdminUser | undefined>(undefined)
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
                  {!u.disabledAt && (
                    <button className="danger-button" onClick={() => setDisableTarget(u)}>
                      無効化
                    </button>
                  )}
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
            setCreatedPassword({ username, password })
            reload()
          }}
        />
      )}

      {createdPassword && (
        <Modal title="ユーザーを作成しました" onClose={() => setCreatedPassword(undefined)}>
          <p className="password-warning">
            この仮パスワードはこの画面を閉じると二度と表示できません。必ずこの場でコピーし、
            利用者に安全な方法で伝達してください。
          </p>
          <p>
            ユーザー名: <strong>{createdPassword.username}</strong>
          </p>
          <code className="password-display" ref={passwordRef}>
            {createdPassword.password}
          </code>
          <button onClick={() => void copyPassword(createdPassword.password)}>📋 コピー</button>
        </Modal>
      )}

      {summaryTarget && (
        <UserSummaryModal user={summaryTarget} onClose={() => setSummaryTarget(undefined)} />
      )}

      {disableTarget && (
        <ConfirmDialog
          title="ユーザーの無効化"
          description={`「${disableTarget.username}」を無効化します。この操作は元に戻せません。`}
          confirmLabel="無効化する"
          onConfirm={async () => {
            try {
              await disableAdminUser(disableTarget.id)
              showToast('success', '無効化しました。')
              setDisableTarget(undefined)
              reload()
            } catch (err) {
              showToast('error', err instanceof ApiError ? err.message : '操作に失敗しました。')
            }
          }}
          onClose={() => setDisableTarget(undefined)}
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
