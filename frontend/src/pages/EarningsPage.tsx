// 決算情報ページ - 一覧。設計書9.2節 + 改善点まとめ資料 対応。
//
// 改善点資料への対応:
//   - 列順序: 評価は重要度が低いため、公開日時の後・タイトルの前に移動
//   - 時間フィルタ: from/toの区別を明示し、カレンダー入力(type="date")のみを受け付ける
//     (テキストでの直接入力を許さず、picker指定のみにすることで見づらさを回避)
//   - 公式情報へのボタンは分かりやすい配色にする(以前は黒背景に青文字で視認性が低かった)
//   - 詳細モーダルに公開日時・評価も表示する
//   - 検索ボタンを廃止し、入力に応じて都度検索する(入力ごとに再検索されるのは仕様として妥当なため、
//     「ボタンがあるのに機能しない」不整合を解消する方向で統一する)

import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAsync } from '../hooks/useAsync'
import { fetchEarnings } from '../api/earnings'
import { formatDateTime } from '../utils/format'
import { AtCoderPager } from '../components/common/Pager'
import { EmptyState, LoadingRow, ErrorState } from '../components/common/States'
import { Modal } from '../components/common/Modal'
import type { Earnings, EarningsEvaluation } from '../types/api'

const EVALUATION_LABEL: Record<EarningsEvaluation, string> = {
  positive: 'ポジティブ',
  neutral: 'ニュートラル',
  negative: 'ネガティブ',
  unrated: '未評価',
}

function useDebounced<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value)
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs)
    return () => clearTimeout(timer)
  }, [value, delayMs])
  return debounced
}

export function EarningsPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const page = Number(searchParams.get('page') ?? '1')

  const [ticker, setTicker] = useState(searchParams.get('ticker') ?? '')
  const [companyName, setCompanyName] = useState(searchParams.get('company_name') ?? '')
  const [evaluation, setEvaluation] = useState(searchParams.get('evaluation') ?? '')
  const [from, setFrom] = useState(searchParams.get('from') ?? '')
  const [to, setTo] = useState(searchParams.get('to') ?? '')
  const [viewMode, setViewMode] = useState<'table' | 'card'>('table')
  const [detail, setDetail] = useState<Earnings | undefined>(undefined)

  const debouncedTicker = useDebounced(ticker, 300)
  const debouncedCompanyName = useDebounced(companyName, 300)

  const filters = useMemo(
    () => ({
      ticker: debouncedTicker || undefined,
      company_name: debouncedCompanyName || undefined,
      evaluation: evaluation || undefined,
      from: from || undefined,
      to: to || undefined,
    }),
    [debouncedTicker, debouncedCompanyName, evaluation, from, to],
  )

  useEffect(() => {
    const next = new URLSearchParams()
    if (filters.ticker) next.set('ticker', filters.ticker)
    if (filters.company_name) next.set('company_name', filters.company_name)
    if (filters.evaluation) next.set('evaluation', filters.evaluation)
    if (filters.from) next.set('from', filters.from)
    if (filters.to) next.set('to', filters.to)
    next.set('page', '1')
    setSearchParams(next, { replace: true })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filters])

  const { data, isLoading, error, reload } = useAsync(
    () => fetchEarnings(filters, page),
    [filters, page],
  )

  const changePage = (nextPage: number) => {
    const next = new URLSearchParams(searchParams)
    next.set('page', String(nextPage))
    setSearchParams(next)
  }

  return (
    <div>
      <PageHeader icon="📈" title="決算情報" />

      <div className="filters">
        <input placeholder="証券コード" value={ticker} onChange={(e) => setTicker(e.target.value)} />
        <input placeholder="銘柄名" value={companyName} onChange={(e) => setCompanyName(e.target.value)} />
        <select value={evaluation} onChange={(e) => setEvaluation(e.target.value)}>
          <option value="">すべての評価</option>
          <option value="positive">ポジティブ</option>
          <option value="neutral">ニュートラル</option>
          <option value="negative">ネガティブ</option>
          <option value="unrated">未評価</option>
        </select>
        <label style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: 6, margin: 0 }}>
          <span style={{ color: 'var(--text-muted)', fontSize: 13 }}>公開日 from</span>
          <input type="date" value={from} onChange={(e) => setFrom(e.target.value)} style={{ width: 'auto' }} />
        </label>
        <label style={{ display: 'flex', flexDirection: 'row', alignItems: 'center', gap: 6, margin: 0 }}>
          <span style={{ color: 'var(--text-muted)', fontSize: 13 }}>to</span>
          <input type="date" value={to} onChange={(e) => setTo(e.target.value)} style={{ width: 'auto' }} />
        </label>

        <div className="view-switch">
          <button className={viewMode === 'table' ? 'active' : ''} onClick={() => setViewMode('table')}>
            テーブル
          </button>
          <button className={viewMode === 'card' ? 'active' : ''} onClick={() => setViewMode('card')}>
            カード
          </button>
        </div>
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && viewMode === 'table' && (
        <table>
          <thead>
            <tr>
              <th>公開日時</th>
              <th>評価</th>
              <th>タイトル</th>
              <th>証券コード</th>
              <th>銘柄名</th>
            </tr>
          </thead>
          <tbody>
            {isLoading && <LoadingRow colSpan={5} />}
            {!isLoading && data?.items.length === 0 && (
              <tr>
                <td colSpan={5}>
                  <EmptyState message="条件に一致する決算情報がありません" />
                </td>
              </tr>
            )}
            {data?.items.map((item) => (
              <tr
                key={item.id}
                className={`evaluation-row ${item.evaluation}`}
                onClick={() => setDetail(item)}
                style={{ cursor: 'pointer' }}
              >
                <td>{formatDateTime(item.publishedAt)}</td>
                <td>{EVALUATION_LABEL[item.evaluation]}</td>
                <td>{item.title}</td>
                <td>{item.ticker}</td>
                <td>{item.companyName}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {!error && viewMode === 'card' && (
        <div className="earning-cards">
          {isLoading && (
            <p className="loading">
              <i /> 読み込み中...
            </p>
          )}
          {!isLoading && data?.items.length === 0 && (
            <EmptyState message="条件に一致する決算情報がありません" />
          )}
          {data?.items.map((item) => (
            <button
              key={item.id}
              className={`earning-card ${item.evaluation}`}
              onClick={() => setDetail(item)}
            >
              <small>{formatDateTime(item.publishedAt)} ・ {EVALUATION_LABEL[item.evaluation]}</small>
              <strong>{item.title}</strong>
              <span>{item.ticker} {item.companyName}</span>
            </button>
          ))}
        </div>
      )}

      {data && data.totalPages > 1 && (
        <AtCoderPager currentPage={page} totalPages={data.totalPages} onChange={changePage} />
      )}

      {detail && <EarningsDetailModal item={detail} onClose={() => setDetail(undefined)} />}
    </div>
  )
}

function EarningsDetailModal({ item, onClose }: { item: Earnings; onClose: () => void }) {
  return (
    <Modal title={item.title} onClose={onClose}>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 16 }}>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>公開日時</small>
          <div>{formatDateTime(item.publishedAt)}</div>
        </div>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>評価</small>
          <div>{EVALUATION_LABEL[item.evaluation]}</div>
        </div>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>証券コード</small>
          <div>{item.ticker}</div>
        </div>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>銘柄名</small>
          <div>{item.companyName}</div>
        </div>
      </div>

      <p style={{ whiteSpace: 'pre-line' }}>{item.summary}</p>

      <a
        href={item.url}
        target="_blank"
        rel="noreferrer"
        className="primary"
        style={{
          display: 'inline-block',
          textDecoration: 'none',
          marginTop: 16,
          padding: '10px 18px',
          borderRadius: 6,
        }}
      >
        公式情報を開く ↗
      </a>
    </Modal>
  )
}
