// ログ閲覧画面。設計書9.12節対応。error/warnは色分けし、fieldsは折りたたまず横スクロールで常時展開する。

import { useSearchParams } from 'react-router-dom'
import { useAsync } from '../../hooks/useAsync'
import { fetchAdminLogs } from '../../api/admin'
import { formatDateTime } from '../../utils/format'
import { AtCoderPager } from '../../components/common/Pager'
import { EmptyState, LoadingRow, ErrorState } from '../../components/common/States'

export function AdminLogsTab() {
  const [searchParams, setSearchParams] = useSearchParams()
  const level = searchParams.get('level') ?? ''
  const process = searchParams.get('process') ?? ''
  const from = searchParams.get('from') ?? ''
  const to = searchParams.get('to') ?? ''
  const page = Number(searchParams.get('page') ?? '1')

  const { data, isLoading, error, reload } = useAsync(
    () => fetchAdminLogs({ level: level || undefined, process: process || undefined, from: from || undefined, to: to || undefined }, page),
    [level, process, from, to, page],
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
        <select value={level} onChange={(e) => updateParams({ level: e.target.value, page: '1' })}>
          <option value="">すべてのlevel</option>
          <option value="trace">trace</option>
          <option value="debug">debug</option>
          <option value="info">info</option>
          <option value="warn">warn</option>
          <option value="error">error</option>
        </select>
        <select value={process} onChange={(e) => updateParams({ process: e.target.value, page: '1' })}>
          <option value="">すべてのprocess</option>
          <option value="server">server</option>
          <option value="monitor">monitor</option>
          <option value="notify">notify</option>
        </select>
        <input type="date" value={from} onChange={(e) => updateParams({ from: e.target.value, page: '1' })} style={{ width: 'auto' }} />
        <span style={{ color: 'var(--text-muted)' }}>〜</span>
        <input type="date" value={to} onChange={(e) => updateParams({ to: e.target.value, page: '1' })} style={{ width: 'auto' }} />
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && (
        <div className="log-table">
          <table>
            <thead>
              <tr>
                <th>日時</th>
                <th>level</th>
                <th>process</th>
                <th>target</th>
                <th>message</th>
                <th>fields</th>
              </tr>
            </thead>
            <tbody>
              {isLoading && <LoadingRow colSpan={6} />}
              {!isLoading && data?.items.length === 0 && (
                <tr>
                  <td colSpan={6}>
                    <EmptyState message="条件に一致するログがありません" />
                  </td>
                </tr>
              )}
              {data?.items.map((log) => (
                <tr key={log.id} className={log.level === 'error' ? 'log-error' : log.level === 'warn' ? 'log-warn' : ''}>
                  <td>{formatDateTime(log.timestamp, true)}</td>
                  <td>{log.level}</td>
                  <td>{log.process}</td>
                  <td>{log.target}</td>
                  <td>{log.message ?? '-'}</td>
                  <td style={{ fontFamily: 'var(--font-body)', fontSize: 12 }}>
                    {Object.keys(log.fields).length > 0 ? JSON.stringify(log.fields) : '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
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
