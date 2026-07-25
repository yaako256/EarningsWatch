// グループ詳細ページ。設計書7.2節・9.5節・9.6節 + 改善点まとめ資料 対応。
//
// 改善点資料への対応:
//   - タブの順番: フィルタをメイン機能とするため左側に配置し、タブ未指定時はfiltersにリダイレクトする
//     (設計書7.2節では「settings」がデフォルトとされていたが、今回のフィードバックにより
//      「フィルタをメインにしたい」という明確な変更希望があったため、デフォルト遷移先も合わせて変更する)
//   - 「一覧に戻る」ボタンをページ見出し付近に追加(以前はサイドバー経由でしか戻れなかった)

import { Link, Navigate, Outlet, useParams, useLocation } from 'react-router-dom'
import { useAsync } from '../../hooks/useAsync'
import { fetchGroups } from '../../api/groups'
import { ErrorState } from '../../components/common/States'

export function GroupDetailLayout() {
  const { groupId } = useParams<{ groupId: string }>()
  const location = useLocation()
  const { data: groups, isLoading, error, reload } = useAsync(fetchGroups, [])

  const group = groups?.find((g) => g.id === groupId)

  if (isLoading) {
    return (
      <p className="loading">
        <i /> 読み込み中...
      </p>
    )
  }
  if (error) return <ErrorState message={error} onRetry={reload} />
  if (!group) return <ErrorState message="グループが見つかりませんでした。" />

  const activeTab = location.pathname.endsWith('/settings') ? 'settings' : 'filters'

  return (
    <div>
      <div className="page-header">
        <div>
          <Link to="/groups" className="link" style={{ fontSize: 13 }}>
            ← グループ一覧に戻る
          </Link>
          <h1 style={{ marginTop: 8 }}>
            <span className="page-icon">{group.medium === 'discord' ? '💬' : '#️⃣'}</span>
            {group.name}
          </h1>
        </div>
      </div>

      <div className="tabs">
        <Link to={`/groups/${groupId}/filters`}>
          <button className={activeTab === 'filters' ? 'active' : ''}>フィルタ</button>
        </Link>
        <Link to={`/groups/${groupId}/settings`}>
          <button className={activeTab === 'settings' ? 'active' : ''}>設定</button>
        </Link>
      </div>

      <Outlet context={{ group, reloadGroups: reload }} />
    </div>
  )
}

export function GroupDetailIndexRedirect() {
  const { groupId } = useParams<{ groupId: string }>()
  return <Navigate to={`/groups/${groupId}/filters`} replace />
}
