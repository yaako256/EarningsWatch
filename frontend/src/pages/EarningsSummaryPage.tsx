// 決算情報 - サマリ画面。設計書9.3節対応。棒グラフには必ず縦軸目盛りを表示する(改善点対応)。

import { useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAsync } from '../hooks/useAsync'
import { fetchEarningsSummary } from '../api/earnings'
import { BarChart } from '../components/common/BarChart'
import { ErrorState } from '../components/common/States'

const RANGE_OPTIONS: { key: string; label: string; days: number | null }[] = [
  { key: 'today', label: '今日', days: 0 },
  { key: 'yesterday', label: '昨日', days: 1 },
  { key: '1w', label: '1週間', days: 7 },
  { key: '1m', label: '1か月', days: 30 },
  { key: '3m', label: '3か月', days: 90 },
  { key: '6m', label: '6か月', days: 180 },
  { key: '1y', label: '1年', days: 365 },
  { key: 'all', label: '全期間', days: null },
]

function rangeToFromTo(days: number | null): { from?: string; to?: string } {
  if (days === null) return {}
  const now = new Date()
  const to = now.toISOString().slice(0, 10)
  const from = new Date(now.getTime() - days * 86400_000).toISOString().slice(0, 10)
  return { from, to }
}

export function EarningsSummaryPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const range = searchParams.get('range') ?? '1m'

  const { from, to } = useMemo(() => {
    const option = RANGE_OPTIONS.find((o) => o.key === range) ?? RANGE_OPTIONS[3]
    return rangeToFromTo(option.days)
  }, [range])

  const { data, isLoading, error, reload } = useAsync(
    () => fetchEarningsSummary(from, to),
    [from, to],
  )

  const chartData = (data?.dailyCounts ?? []).map((d) => ({ label: d.dateJst.slice(5), value: d.count }))

  return (
    <div>
      <PageHeader icon="📈" title="決算情報 - サマリ" />

      <div className="range-buttons">
        {RANGE_OPTIONS.map((option) => (
          <button
            key={option.key}
            className={range === option.key ? 'active' : ''}
            onClick={() => setSearchParams({ range: option.key })}
          >
            {option.label}
          </button>
        ))}
      </div>

      {error && <ErrorState message={error} onRetry={reload} />}

      {!error && isLoading && (
        <p className="loading">
          <i /> 読み込み中...
        </p>
      )}

      {!error && !isLoading && <BarChart data={chartData} />}
    </div>
  )
}
