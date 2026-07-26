// 決算情報ページ - 一覧。設計書9.2節 + 改善点まとめ資料(Ver1・Ver2) 対応。
//
// Ver2改善点への対応:
//   - エクスポートボタンが消失していたため復旧
//   - 評価ラベルのtypo("Nuetral"→"Neutral")を修正
//   - 評価色は行の背景色・専用バッジクラスと連動させ、行との対応が伝わるようにする
//   - 詳細モーダル・カードは「証券コード/銘柄名」を上、「公開日時/評価」を下の順序に変更

import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAsync } from '../hooks/useAsync'
import { fetchEarnings } from '../api/earnings'
import { formatDateTime, exportFileName } from '../utils/format'
import { AtCoderPager } from '../components/common/Pager'
import { EmptyState, LoadingRow, ErrorState } from '../components/common/States'
import { Modal } from '../components/common/Modal'
import { downloadFile, qs } from '../api/client'
import { useToast } from '../contexts/ToastContext'
import type { Earnings, EarningsEvaluation } from '../types/api'

const EVALUATION_LABEL: Record<EarningsEvaluation, string> = {
  positive: 'Positive',
  neutral: 'Neutral',
  negative: 'Negative',
  unrated: 'Unrated',
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
  const { showToast } = useToast()

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

  // API設計書7.3節・Rust側 ExportEarningsQuery より:
  // /earnings/export は format(必須) に加えて ticker/company_name/evaluation/from/to の
  // 絞り込みクエリも受け付ける。画面で選択中の条件をそのままエクスポートにも反映させる。
  const handleExport = async () => {
    try {
      const query = qs({ ...filters, format: 'xlsx' })
      await downloadFile(`/earnings/export?${query}`, exportFileName('earnings.xlsx'))
    } catch {
      showToast('error', 'エクスポートに失敗しました。')
    }
  }

  return (
    <div>
      <PageHeader
        icon="📈"
        title="決算情報"
        actions={<button onClick={handleExport}>⇩ エクスポート</button>}
      />

      <div className="filters">
        <input placeholder="証券コード(完全一致)" value={ticker} onChange={(e) => setTicker(e.target.value)} />
        <input placeholder="銘柄名" value={companyName} onChange={(e) => setCompanyName(e.target.value)} />
        <select value={evaluation} onChange={(e) => setEvaluation(e.target.value)}>
          <option value="">すべての評価</option>
          <option value="positive">Positive</option>
          <option value="neutral">Neutral</option>
          <option value="negative">Negative</option>
          <option value="unrated">Unrated</option>
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
              <th>証券コード</th>
              <th>銘柄名</th>
              <th>公開日時</th>
              <th>評価</th>
              <th>タイトル</th>
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
                <td>{item.ticker}</td>
                <td>{item.companyName}</td>
                <td>{formatDateTime(item.publishedAt)}</td>
                <td>
                  <EvaluationBadge evaluation={item.evaluation} />
                </td>
                <td>{item.title}</td>
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
              <span>{item.ticker} {item.companyName}</span>
              <strong>{item.title}</strong>
              <small>
                {formatDateTime(item.publishedAt)} ・ <EvaluationBadge evaluation={item.evaluation} />
              </small>
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

function EvaluationBadge({ evaluation }: { evaluation: EarningsEvaluation }) {
  return <span className={`badge eval-${evaluation}`}>{EVALUATION_LABEL[evaluation]}</span>
}

function EarningsDetailModal({ item, onClose }: { item: Earnings; onClose: () => void }) {
  return (
    <Modal title={item.title} onClose={onClose}>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, marginBottom: 16 }}>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>証券コード</small>
          <div>{item.ticker}</div>
        </div>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>銘柄名</small>
          <div>{item.companyName}</div>
        </div>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>公開日時</small>
          <div>{formatDateTime(item.publishedAt)}</div>
        </div>
        <div>
          <small style={{ color: 'var(--text-muted)' }}>評価</small>
          <div>
            <EvaluationBadge evaluation={item.evaluation} />
          </div>
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
