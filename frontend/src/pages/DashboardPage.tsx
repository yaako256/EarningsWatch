// ダッシュボード(一般ユーザ用)。設計書9.1節 + Ver1からの改善点まとめ資料 対応。
//
// 改善点資料への対応:
//   - グループ数→その内訳のDiscord/Slack、フィルタ数→その内訳のユニーク証券コード/銘柄、という
//     親子関係が伝わるカード構成にする(以前は全カードが横一列で並び、関係が読み取れなかった)
//   - 一時停止・webhook未設定は「意図的な操作」であるため警告色で強調しない(通常のカード扱いにする)
//   - 直近送信失敗のみ、実際に見てもらう必要がある異常なので控えめな警告色に留める

import { Link } from 'react-router-dom'
import { PageHeader } from '../components/common/PageHeader'
import { useAsync } from '../hooks/useAsync'
import { fetchDashboard } from '../api/deliveries'
import { formatDateTime, formatNumber } from '../utils/format'
import { EmptyState, ErrorState } from '../components/common/States'

export function DashboardPage() {
  const { data, isLoading, error, reload } = useAsync(fetchDashboard, [])

  return (
    <div>
      <PageHeader icon="📊" title="ダッシュボード" />

      {isLoading && (
        <p className="loading">
          <i /> 読み込み中...
        </p>
      )}
      {error && <ErrorState message={error} onRetry={reload} />}

      {data && (
        <>
          <h2>グループ</h2>
          <div className="cards">
            <div className="card">
              <small>グループ数(全体)</small>
              <strong>{formatNumber(data.groupCount)}</strong>
            </div>
            <div className="card">
              <small>グループ数(Discord)</small>
              <strong>{formatNumber(data.mediumBreakdown.discord)}</strong>
            </div>
            <div className="card">
              <small>グループ数(Slack)</small>
              <strong>{formatNumber(data.mediumBreakdown.slack)}</strong>
            </div>
            <div className="card">
              <small>一時停止中</small>
              <strong>{formatNumber(data.pausedGroupCount)}</strong>
            </div>
            <div className="card">
              <small>Webhook未設定</small>
              <strong>{formatNumber(data.webhookMissingCount)}</strong>
            </div>
          </div>

          <h2>フィルタ</h2>
          <div className="cards">
            <div className="card">
              <small>フィルタ数(全体)</small>
              <strong>{formatNumber(data.filterCount)}</strong>
            </div>
            <div className="card">
              <small>ユニーク証券コード数</small>
              <strong>{formatNumber(data.uniqueTickerCount)}</strong>
            </div>
            <div className="card">
              <small>ユニーク銘柄数</small>
              <strong>{formatNumber(data.uniqueCompanyNameCount)}</strong>
            </div>
          </div>

          <h2>直近の送信</h2>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 24 }}>
            <div>
              <h3 style={{ fontSize: 14, color: 'var(--text-muted)', margin: '0 0 8px' }}>
                送信成功
              </h3>
              {data.recentSent.length === 0 ? (
                <EmptyState message="直近の送信成功はありません" />
              ) : (
                <table>
                  <tbody>
                    {data.recentSent.map((item) => (
                      <tr key={item.id}>
                        <td>{formatDateTime(item.sentAt)}</td>
                        <td>
                          {item.groupId ? (
                            <Link className="link" to={`/groups/${item.groupId}`}>
                              {item.groupName ?? item.groupId}
                            </Link>
                          ) : (
                            '-'
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>

            <div>
              <h3 style={{ fontSize: 14, color: 'var(--danger)', margin: '0 0 8px' }}>送信失敗</h3>
              {data.recentFailed.length === 0 ? (
                <EmptyState message="直近の送信失敗はありません" />
              ) : (
                <table>
                  <tbody>
                    {data.recentFailed.map((item) => (
                      <tr key={item.id} className="evaluation-row negative">
                        <td>{formatDateTime(item.sentAt)}</td>
                        <td>
                          {item.groupId ? (
                            <Link className="link" to={`/groups/${item.groupId}`}>
                              {item.groupName ?? item.groupId}
                            </Link>
                          ) : (
                            '-'
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}
