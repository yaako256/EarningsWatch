// お知らせ板画面。設計書9.8節 + 改善点まとめ資料(Ver1・Ver2) 対応。
//
// Ver1改善点への対応:
//   - 本文はMarkdownとしてきちんとレンダリングする(以前は#等がそのまま出力されていた)
//   - staticページの並べ替えは、ドラッグ中の行がカーソルに追従し、ドラッグ中もリアルタイムで
//     順序が入れ替わるように動作させる
//   - 一覧のヘッダー(タイトル/状態/作成者の列)のずれを修正する
// Ver2改善点への対応:
//   - タブ(blog/static)の状態をURLクエリに保持し、固定ページ詳細から戻ったときに
//     固定ページ一覧に戻れるようにする(以前はblogタブに固定で戻ってしまっていた)
//   - 一覧上の「編集」は別ページに遷移せず、一覧の上にモーダルを開いたまま編集できるようにする

import { useMemo, useState } from 'react'
import type { DragEvent } from 'react'
import { Link, Route, Routes, useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAuth } from '../contexts/AuthContext'
import { useAsync } from '../hooks/useAsync'
import {
  fetchPageList,
  fetchPageDetail,
  createPage,
  updatePage,
  deletePage,
  reorderPage,
} from '../api/pages'
import { renderMarkdown } from '../utils/renderMarkdown'
import { formatDateTime } from '../utils/format'
import { EmptyState, LoadingRow, ErrorState } from '../components/common/States'
import { Modal } from '../components/common/Modal'
import { ConfirmDialog } from '../components/common/ConfirmDialog'
import { useToast } from '../contexts/ToastContext'
import { ApiError } from '../api/client'
import type { PageDetail, PageListItem, PageType } from '../types/api'

export function AnnouncementsPage() {
  return (
    <Routes>
      <Route index element={<AnnouncementList />} />
      <Route path=":id" element={<AnnouncementDetail />} />
    </Routes>
  )
}

function AnnouncementList() {
  const { user } = useAuth()
  const isAdmin = user?.role === 'admin'
  const [searchParams, setSearchParams] = useSearchParams()
  const tab: PageType = searchParams.get('type') === 'static' ? 'static' : 'blog'
  const setTab = (next: PageType) => setSearchParams({ type: next })
  const { showToast } = useToast()

  const { data, isLoading, error, reload } = useAsync(() => fetchPageList(tab), [tab])
  const visibleItems = useMemo(
    () => (isAdmin ? data : data?.filter((item) => item.isPublished)),
    [data, isAdmin],
  )

  const [showCreate, setShowCreate] = useState(false)
  const [editTarget, setEditTarget] = useState<PageListItem | undefined>(undefined)
  const [deleteTarget, setDeleteTarget] = useState<PageListItem | undefined>(undefined)
  const [orderedItems, setOrderedItems] = useState<PageListItem[] | undefined>(undefined)
  const [draggingId, setDraggingId] = useState<string | undefined>(undefined)
  const [dragOverId, setDragOverId] = useState<string | undefined>(undefined)

  const items = tab === 'static' && orderedItems ? orderedItems : visibleItems

  const handleDragStart = (id: string) => {
    setDraggingId(id)
    setOrderedItems(visibleItems ? [...visibleItems] : undefined)
  }

  const handleDragOver = (e: DragEvent<HTMLTableRowElement>, overId: string) => {
    e.preventDefault()
    if (!draggingId || draggingId === overId || !orderedItems) return
    setDragOverId(overId)
    const fromIndex = orderedItems.findIndex((i) => i.id === draggingId)
    const toIndex = orderedItems.findIndex((i) => i.id === overId)
    if (fromIndex === -1 || toIndex === -1) return
    const next = [...orderedItems]
    const [moved] = next.splice(fromIndex, 1)
    next.splice(toIndex, 0, moved)
    setOrderedItems(next)
  }

  const handleDrop = async () => {
    if (!draggingId || !orderedItems) return
    const index = orderedItems.findIndex((i) => i.id === draggingId)
    const prevOrder = orderedItems[index - 1]?.displayOrder ?? 0
    const nextOrder = orderedItems[index + 1]?.displayOrder ?? prevOrder + 2
    const newOrder = (prevOrder + nextOrder) / 2

    setDraggingId(undefined)
    setDragOverId(undefined)
    try {
      await reorderPage(draggingId, newOrder)
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '並べ替えに失敗しました。')
      setOrderedItems(undefined)
    }
  }

  return (
    <div>
      <PageHeader
        icon="📌"
        title="お知らせ板"
        actions={isAdmin ? <button className="primary" onClick={() => setShowCreate(true)}>+ 新規作成</button> : undefined}
      />

      <div className="tabs">
        <button className={tab === 'blog' ? 'active' : ''} onClick={() => setTab('blog')}>
          お知らせ記事
        </button>
        <button className={tab === 'static' ? 'active' : ''} onClick={() => setTab('static')}>
          固定ページ
        </button>
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <table>
          <thead>
            <tr>
              {tab === 'static' && isAdmin && <th style={{ width: 32 }}></th>}
              <th>タイトル</th>
              {isAdmin && <th>状態</th>}
              <th>作成者</th>
              <th>更新日時</th>
              {isAdmin && <th></th>}
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={5} />}
            {!isLoading && items?.length === 0 && (
              <tr>
                <td colSpan={5}>
                  <EmptyState message="お知らせはまだありません" />
                </td>
              </tr>
            )}
            {items?.map((item) => (
              <tr
                key={item.id}
                className={item.id === dragOverId ? 'drag-over' : item.id === draggingId ? 'dragging' : ''}
                draggable={tab === 'static' && isAdmin}
                onDragStart={() => handleDragStart(item.id)}
                onDragOver={(e) => handleDragOver(e, item.id)}
                onDrop={handleDrop}
                onDragEnd={() => {
                  setDraggingId(undefined)
                  setDragOverId(undefined)
                }}
              >
                {tab === 'static' && isAdmin && (
                  <td>
                    <span className="drag-handle">⠿</span>
                  </td>
                )}
                <td>
                  <Link className="link" to={`/announcements/${item.id}?type=${tab}`}>
                    {item.title}
                  </Link>
                </td>
                {isAdmin && (
                  <td>
                    <span className={`badge ${item.isPublished ? 'good' : ''}`}>
                      {item.isPublished ? '公開中' : '下書き'}
                    </span>
                  </td>
                )}
                <td>{item.authorUsername}</td>
                <td>{formatDateTime(item.updatedAt)}</td>
                {isAdmin && (
                  <td>
                    <div className="cell-actions">
                      <button onClick={() => setEditTarget(item)}>編集</button>
                      <button className="danger-button" onClick={() => setDeleteTarget(item)}>
                        削除
                      </button>
                    </div>
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {showCreate && (
        <AnnouncementEditModal
          type={tab}
          onClose={() => setShowCreate(false)}
          onSaved={() => {
            setShowCreate(false)
            reload()
          }}
        />
      )}

      {/* Ver2改善点対応: 編集は一覧ページ上のモーダルで完結させ、詳細ページへは遷移しない */}
      {editTarget && (
        <AnnouncementEditModalById
          id={editTarget.id}
          type={tab}
          onClose={() => setEditTarget(undefined)}
          onSaved={() => {
            setEditTarget(undefined)
            reload()
          }}
        />
      )}

      {deleteTarget && (
        <ConfirmDialog
          title="削除の確認"
          description={`「${deleteTarget.title}」を削除します。元に戻せません。`}
          confirmLabel="削除する"
          onConfirm={async () => {
            try {
              await deletePage(deleteTarget.id)
              showToast('success', '削除しました。')
              setDeleteTarget(undefined)
              reload()
            } catch (err) {
              showToast('error', err instanceof ApiError ? err.message : '削除に失敗しました。')
            }
          }}
          onClose={() => setDeleteTarget(undefined)}
        />
      )}
    </div>
  )
}

/** 一覧の「編集」から呼ばれる版。詳細データを取得してからモーダルを表示する。 */
function AnnouncementEditModalById({
  id,
  type,
  onClose,
  onSaved,
}: {
  id: string
  type: PageType
  onClose: () => void
  onSaved: () => void
}) {
  const { data, isLoading, error } = useAsync(() => fetchPageDetail(id), [id])

  if (isLoading) {
    return (
      <Modal title="編集" onClose={onClose}>
        <p className="loading">
          <i /> 読み込み中...
        </p>
      </Modal>
    )
  }
  if (error || !data) {
    return (
      <Modal title="編集" onClose={onClose}>
        <ErrorState message={error ?? '取得に失敗しました。'} />
      </Modal>
    )
  }

  return <AnnouncementEditModal type={type} initial={data} onClose={onClose} onSaved={onSaved} />
}

function AnnouncementDetail() {
  const { id } = useParams<{ id: string }>()
  const [searchParams] = useSearchParams()
  // 一覧のどのタブから遷移してきたかをクエリで保持し、「一覧に戻る」で同じタブに戻れるようにする
  // (Ver2改善点対応: 以前は固定ページ詳細から戻ると常にblogタブに戻ってしまっていた)。
  const originTab: PageType = searchParams.get('type') === 'static' ? 'static' : 'blog'
  const navigate = useNavigate()

  const { data, isLoading, error, reload } = useAsync(() => fetchPageDetail(id!), [id])

  if (isLoading) {
    return (
      <p className="loading">
        <i /> 読み込み中...
      </p>
    )
  }
  if (error || !data) return <ErrorState message={error ?? '見つかりませんでした。'} onRetry={reload} />

  return (
    <div>
      <button
        className="link"
        onClick={() => navigate(`/announcements?type=${originTab}`)}
        style={{ marginBottom: 16 }}
      >
        ← 一覧に戻る
      </button>
      <h1>{data.title}</h1>
      <p className="meta">
        {formatDateTime(data.updatedAt)} ・ {data.authorUsername}
      </p>
      <div className="markdown-body" dangerouslySetInnerHTML={{ __html: renderMarkdown(data.contentMarkdown) }} />
    </div>
  )
}

function AnnouncementEditModal({
  type,
  initial,
  onClose,
  onSaved,
}: {
  type: PageType
  initial?: PageDetail
  onClose: () => void
  onSaved: () => void
}) {
  const { showToast } = useToast()
  const [title, setTitle] = useState(initial?.title ?? '')
  const [content, setContent] = useState(initial?.contentMarkdown ?? '')
  const [isPublished, setIsPublished] = useState(initial?.isPublished ?? false)
  const [titleError, setTitleError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async () => {
    setTitleError('')
    if (title.trim() === '') {
      setTitleError('タイトルを入力してください。')
      return
    }
    setIsSubmitting(true)
    try {
      if (initial) {
        await updatePage(initial.id, { title, contentMarkdown: content, isPublished })
      } else {
        await createPage({ type, title, contentMarkdown: content, displayOrder: null, isPublished })
      }
      showToast('success', '保存しました。')
      onSaved()
    } catch (err) {
      if (err instanceof ApiError && err.code === 'invalid_request') {
        setTitleError(err.message)
      } else {
        showToast('error', err instanceof ApiError ? err.message : '保存に失敗しました。')
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Modal title={initial ? '編集' : '新規作成'} onClose={onClose} wide>
      <label>
        タイトル<span className="required-mark">*</span>
        <input value={title} onChange={(e) => setTitle(e.target.value)} autoFocus />
      </label>
      {titleError && <p className="inline-error">{titleError}</p>}

      <label>
        本文(Markdown)
        <textarea rows={12} value={content} onChange={(e) => setContent(e.target.value)} />
      </label>

      <div className="check-row">
        <input type="checkbox" checked={isPublished} onChange={(e) => setIsPublished(e.target.checked)} />
        <span>公開する</span>
      </div>

      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>キャンセル</button>
        <button className="primary" disabled={isSubmitting} onClick={handleSubmit}>
          {isSubmitting ? '保存中...' : '保存する'}
        </button>
      </div>
    </Modal>
  )
}
