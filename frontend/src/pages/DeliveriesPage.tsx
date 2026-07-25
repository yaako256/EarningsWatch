// 送信キュー/履歴ページ。設計書9.7節対応。

import { useState } from 'react'
import { NavLink, Route, Routes, useSearchParams } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAsync } from '../hooks/useAsync'
import { fetchNotifyQueue, fetchNotifyHistory } from '../api/deliveries'
import { fetchGroups } from '../api/groups'
import { formatDateTime } from '../utils/format'
import { SimplePager, AtCoderPager } from '../components/common/Pager'
import { EmptyState, LoadingRow, ErrorState } from '../components/common/States'
import type { NotifyStatus } from '../types/api'

const STATUS_LABEL: Record<NotifyStatus, string> = { ready: '送信待ち', sent: '送信済み', failed: '失敗' }

export function DeliveriesPage() {
  return (
    <div>
      <PageHeader icon="📬" title="送信キュー/履歴" />
      <div className="tabs">
        <NavLink to="/deliveries" end>
          {({ isActive }) => <button className={isActive ? 'active' : ''}>キュー</button>}
        </NavLink>
        <NavLink to="/deliveries/history">
          {({ isActive }) => <button className={isActive ? 'active' : ''}>履歴</button>}
        </NavLink>
      </div>
      <Routes>
        <Route index element={<QueueTab />} />
        <Route path="history" element={<HistoryTab />} />
      </Routes>
    </div>
  )
}

function QueueTab() {
  const [searchParams, setSearchParams] = useSearchParams()
  const status = searchParams.get('status') ?? ''
  const page = Number(searchParams.get('page') ?? '1')

  const { data, isLoading, error, reload } = useAsync(
    () => fetchNotifyQueue(status || undefined, page),
    [status, page],
  )

  const updateParams = (next: Record<string, string>) => {
    const params = new URLSearchParams(searchParams)
    Object.entries(next).forEach(([key, value]) => {
      if (value) params.set(key, value)
      else params.delete(key)
    })
    setSearchParams(params)
  }

  return (
    <div>
      <div className="filters">
        <select value={status} onChange={(e) => updateParams({ status: e.target.value, page: '1' })}>
          <option value="">すべての状態</option>
          <option value="ready">送信待ち</option>
          <option value="sent">送信済み</option>
          <option value="failed">失敗</option>
        </select>
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <table>
          <thead>
            <tr>
              <th>公開日時</th>
              <th>証券コード</th>
              <th>銘柄名</th>
              <th>タイトル</th>
              <th>状態</th>
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={5} />}
            {!isLoading && data?.items.length === 0 && (
              <tr>
                <td colSpan={5}>
                  <EmptyState message="キューは空です" />
                </td>
              </tr>
            )}
            {data?.items.map((item) => (
              <tr key={item.id}>
                <td>{formatDateTime(item.publishedAt)}</td>
                <td>{item.ticker}</td>
                <td>{item.companyName}</td>
                <td>{item.title}</td>
                <td>
                  <span className={`badge ${item.status === 'sent' ? 'good' : ''}`}>
                    {STATUS_LABEL[item.status]}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {data && (
        <SimplePager
          currentPage={page}
          hasNext={data.items.length === data.perPage}
          onChange={(p) => updateParams({ page: String(p) })}
        />
      )}
    </div>
  )
}

function HistoryTab() {
  const [searchParams, setSearchParams] = useSearchParams()
  const groupId = searchParams.get('group_id') ?? ''
  const status = searchParams.get('status') ?? ''
  const page = Number(searchParams.get('page') ?? '1')

  const { data: groups } = useAsync(fetchGroups, [])
  const { data, isLoading, error, reload } = useAsync(
    () => fetchNotifyHistory(groupId || undefined, status || undefined, page),
    [groupId, status, page],
  )

  const updateParams = (next: Record<string, string>) => {
    const params = new URLSearchParams(searchParams)
    Object.entries(next).forEach(([key, value]) => {
      if (value) params.set(key, value)
      else params.delete(key)
    })
    setSearchParams(params)
  }

  return (
    <div>
      <div className="filters">
        <select value={groupId} onChange={(e) => updateParams({ group_id: e.target.value, page: '1' })}>
          <option value="">全グループ</option>
          {groups?.map((g) => (
            <option key={g.id} value={g.id}>
              {g.name}
            </option>
          ))}
        </select>
        <select value={status} onChange={(e) => updateParams({ status: e.target.value, page: '1' })}>
          <option value="">すべての状態</option>
          <option value="sent">送信済み</option>
          <option value="failed">失敗</option>
        </select>
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <table>
          <thead>
            <tr>
              <th>送信日時</th>
              <th>グループ</th>
              <th>状態</th>
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={3} />}
            {!isLoading && data?.items.length === 0 && (
              <tr>
                <td colSpan={3}>
                  <EmptyState message="履歴がありません" />
                </td>
              </tr>
            )}
            {data?.items.map((item) => (
              <tr key={item.id}>
                <td>{formatDateTime(item.sentAt)}</td>
                <td>{item.groupName ?? '-'}</td>
                <td>
                  <span className={`badge ${item.status === 'sent' ? 'good' : ''}`}>
                    {STATUS_LABEL[item.status]}
                  </span>
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
          onChange={(p) => updateParams({ page: String(p) })}
        />
      )}
    </div>
  )
}
