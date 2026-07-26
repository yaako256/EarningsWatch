// 管理者ダッシュボード。設計書9.9節: monitor用・notify用のグラフを完全に分離して表示する
// (実行時間のスケールが大きく異なるため、1つのグラフに重ねない)。

import { useAsync } from '../../hooks/useAsync'
import { fetchAdminDashboard } from '../../api/admin'
import { formatDateTime, formatNumber, formatRate } from '../../utils/format'
import { BarChart } from '../../components/common/BarChart'
import { ErrorState } from '../../components/common/States'

export function AdminDashboardTab() {
  const { data, isLoading, error, reload } = useAsync(fetchAdminDashboard, [])

  if (isLoading) {
    return (
      <p className="loading">
        <i /> 読み込み中...
      </p>
    )
  }
  if (error || !data) return <ErrorState message={error ?? '取得に失敗しました。'} onRetry={reload} />

  const isLowSuccessRate = data.notifySuccessRate !== null && data.notifySuccessRate < 0.9
  const monitorRuns = data.runDurations.filter((r) => r.runType === 'monitor')
  const notifyRuns = data.runDurations.filter((r) => r.runType === 'notify')

  return (
    <div>
      <div className="cards">
        <div className="card">
          <small>総決算件数</small>
          <strong>{formatNumber(data.totalEarningsCount)}</strong>
        </div>
        <div className="card">
          <small>最終監視実行</small>
          <strong className="date">{formatDateTime(data.lastMonitorRunAt) === '-' ? '未実行' : formatDateTime(data.lastMonitorRunAt)}</strong>
        </div>
        <div className={`card ${isLowSuccessRate ? 'muted-emphasis' : ''}`}>
          <small>通知成功率</small>
          <strong style={{ color: isLowSuccessRate ? 'var(--warning)' : undefined }}>
            {formatRate(data.notifySuccessRate)}
          </strong>
        </div>
      </div>

      <h2>監視処理(monitor)の実行時間推移</h2>
      <BarChart
        data={monitorRuns.map((r) => ({ label: formatDateTime(r.runAt).slice(5, 16), value: r.durationMs }))}
      />

      <h2>通知処理(notify)の実行時間推移</h2>
      <BarChart
        data={notifyRuns.map((r) => ({ label: formatDateTime(r.runAt).slice(5, 16), value: r.durationMs }))}
      />
    </div>
  )
}
