// グループ詳細 - フィルタタブ。設計書9.6節 + 改善点まとめ資料 対応。
//
// 改善点資料への対応:
//   - 一括設定は「一括設定するボタン」を押したらトグル(チェックボックス列)が現れる方式に変更
//     (以前はチェックを入れると操作ボタンが生える方式だった)。
//   - 一括操作パネルは、フィルタの下(ツールバーと同じ高さ・右側)に表示し、フィルタ本体の表示位置がずれないようにする
//   - 編集ボタンを左、無効化ボタンを右にする(先に編集を検討する動線を想定)
//   - グループ単位の一括インポート機能を追加

import { useMemo, useState } from 'react'
import { useOutletContext } from 'react-router-dom'
import { useAsync } from '../../hooks/useAsync'
import {
  fetchGroupFilters,
  createFilter,
  updateFilter,
  setFilterEnabled,
  deleteFilter,
  bulkFilterAction,
} from '../../api/filters'
import { importFiltersForGroup } from '../../api/groups'
import { EmptyState, LoadingRow, ErrorState } from '../../components/common/States'
import { Modal } from '../../components/common/Modal'
import { ImportModal } from '../../components/common/ImportModal'
import { ConfirmDialog } from '../../components/common/ConfirmDialog'
import { useToast } from '../../contexts/ToastContext'
import { ApiError, downloadFile } from '../../api/client'
import { exportFileName } from '../../utils/format'
import type { Filter, Group, ImportRow } from '../../types/api'

interface OutletContext {
  group: Group
}

export function GroupFiltersTab() {
  const { group } = useOutletContext<OutletContext>()
  const { showToast } = useToast()
  const { data, isLoading, error, reload } = useAsync(
    () => fetchGroupFilters(group.id).then((page) => page.items),
    [group.id],
  )

  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [showImport, setShowImport] = useState(false)
  const [editTarget, setEditTarget] = useState<Filter | undefined>(undefined)
  const [deleteTarget, setDeleteTarget] = useState<Filter | undefined>(undefined)
  const [bulkMode, setBulkMode] = useState(false)
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [busyId, setBusyId] = useState<string | undefined>(undefined)
  const [isBulkSubmitting, setIsBulkSubmitting] = useState(false)

  const filtered = useMemo(() => {
    if (!data) return []
    const query = search.trim().toLowerCase()
    if (!query) return data
    return data.filter(
      (f) => f.ticker.toLowerCase().includes(query) || f.companyName.toLowerCase().includes(query),
    )
  }, [data, search])

  const toggleSelected = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  const exitBulkMode = () => {
    setBulkMode(false)
    setSelected(new Set())
  }

  const runBulk = async (action: 'enable' | 'disable' | 'delete') => {
    if (selected.size === 0) return
    setIsBulkSubmitting(true)
    try {
      const result = await bulkFilterAction(action, [...selected])
      showToast('success', `${result.updatedCount}件を更新しました。`)
      exitBulkMode()
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '一括操作に失敗しました。')
    } finally {
      setIsBulkSubmitting(false)
    }
  }

  const toggleEnabled = async (filter: Filter) => {
    setBusyId(filter.id)
    try {
      await setFilterEnabled(group.id, filter.id, !filter.enabled)
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
      await deleteFilter(group.id, deleteTarget.id)
      showToast('success', 'フィルタを削除しました。')
      setDeleteTarget(undefined)
      reload()
    } catch (err) {
      showToast('error', err instanceof ApiError ? err.message : '削除に失敗しました。')
    } finally {
      setBusyId(undefined)
    }
  }

  const handleExport = async () => {
    try {
      await downloadFile(`/groups/${group.id}/filters/export`, exportFileName(`${group.name}_filters.xlsx`))
    } catch {
      showToast('error', 'エクスポートに失敗しました。')
    }
  }

  return (
    <div>
      <div className="toolbar">
        <input
          placeholder="証券コード・銘柄名で検索"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <button onClick={() => setShowImport(true)}>⇪ 一括インポート</button>
        <button onClick={handleExport}>⇩ エクスポート</button>
        <button className="primary" onClick={() => setShowCreate(true)}>
          + フィルタを追加
        </button>
        {!bulkMode && <button onClick={() => setBulkMode(true)}>☑ 一括設定する</button>}
      </div>

      {bulkMode && (
        <div className="bulk-panel">
          <span>{selected.size}件選択中</span>
          <button disabled={selected.size === 0 || isBulkSubmitting} onClick={() => runBulk('enable')}>
            有効化
          </button>
          <button disabled={selected.size === 0 || isBulkSubmitting} onClick={() => runBulk('disable')}>
            無効化
          </button>
          <button
            className="danger-button"
            disabled={selected.size === 0 || isBulkSubmitting}
            onClick={() => runBulk('delete')}
          >
            削除
          </button>
          <button onClick={exitBulkMode} style={{ marginLeft: 'auto' }}>
            一括設定を終了
          </button>
        </div>
      )}

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <table>
          <thead>
            <tr>
              {bulkMode && <th style={{ width: 36 }}></th>}
              <th>証券コード</th>
              <th>銘柄名</th>
              <th>メモ</th>
              <th>状態</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={bulkMode ? 6 : 5} />}
            {!isLoading && filtered.length === 0 && (
              <tr>
                <td colSpan={bulkMode ? 6 : 5}>
                  <EmptyState message="フィルタがまだありません" />
                </td>
              </tr>
            )}
            {filtered.map((filter) => (
              <tr key={filter.id}>
                {bulkMode && (
                  <td>
                    <input
                      type="checkbox"
                      checked={selected.has(filter.id)}
                      onChange={() => toggleSelected(filter.id)}
                    />
                  </td>
                )}
                <td>{filter.ticker}</td>
                <td>{filter.companyName}</td>
                <td className="cell-notes" title={filter.notes ?? undefined}>
                  {filter.notes || '-'}
                </td>
                <td>
                  {filter.enabled ? (
                    <span className="badge good">有効</span>
                  ) : (
                    <span className="badge">無効</span>
                  )}
                </td>
                <td>
                  <div className="cell-actions">
                    <button disabled={busyId === filter.id} onClick={() => setEditTarget(filter)}>
                      編集
                    </button>
                    <button disabled={busyId === filter.id} onClick={() => toggleEnabled(filter)}>
                      {filter.enabled ? '無効化' : '有効化'}
                    </button>
                    <button
                      className="danger-button"
                      disabled={busyId === filter.id}
                      onClick={() => setDeleteTarget(filter)}
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

      {(showCreate || editTarget) && (
        <FilterFormModal
          groupId={group.id}
          initial={editTarget}
          onClose={() => {
            setShowCreate(false)
            setEditTarget(undefined)
          }}
          onSaved={() => {
            setShowCreate(false)
            setEditTarget(undefined)
            reload()
          }}
        />
      )}

      {deleteTarget && (
        <ConfirmDialog
          title="フィルタの削除"
          description={`「${deleteTarget.ticker} ${deleteTarget.companyName}」を削除します。`}
          confirmLabel="削除する"
          isSubmitting={busyId === deleteTarget.id}
          onConfirm={handleDelete}
          onClose={() => setDeleteTarget(undefined)}
        />
      )}

      {showImport && (
        <ImportModal
          title={`フィルタの一括インポート(${group.name})`}
          scopeHint="このグループ内のフィルタとして登録されます。"
          onImport={(rows: ImportRow[], dryRun) => importFiltersForGroup(group.id, rows, dryRun)}
          onClose={() => setShowImport(false)}
          onImported={reload}
        />
      )}
    </div>
  )
}

function FilterFormModal({
  groupId,
  initial,
  onClose,
  onSaved,
}: {
  groupId: string
  initial?: Filter
  onClose: () => void
  onSaved: () => void
}) {
  const { showToast } = useToast()
  const [ticker, setTicker] = useState(initial?.ticker ?? '')
  const [companyName, setCompanyName] = useState(initial?.companyName ?? '')
  const [notes, setNotes] = useState(initial?.notes ?? '')
  const [tickerError, setTickerError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmit = async () => {
    setTickerError('')
    if (ticker.trim() === '') {
      setTickerError('証券コードを入力してください。')
      return
    }
    setIsSubmitting(true)
    try {
      const input = { ticker, companyName, notes: notes || null }
      if (initial) {
        await updateFilter(groupId, initial.id, input)
      } else {
        await createFilter(groupId, input)
      }
      showToast('success', '保存しました。')
      onSaved()
    } catch (err) {
      if (err instanceof ApiError && err.code === 'invalid_request') {
        setTickerError(err.message)
      } else {
        showToast('error', err instanceof ApiError ? err.message : '保存に失敗しました。')
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Modal title={initial ? 'フィルタを編集' : 'フィルタを追加'} onClose={onClose}>
      <label>
        証券コード<span className="required-mark">*</span>
        <input value={ticker} onChange={(e) => setTicker(e.target.value)} autoFocus />
      </label>
      {tickerError && <p className="inline-error">{tickerError}</p>}

      <label>
        銘柄名
        <input value={companyName} onChange={(e) => setCompanyName(e.target.value)} />
      </label>
      <label>
        メモ
        <textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} />
      </label>

      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>キャンセル</button>
        <button className="primary" disabled={isSubmitting} onClick={handleSubmit}>
          {isSubmitting ? '保存中...' : '保存する'}
        </button>
      </div>
    </Modal>
  )
}
