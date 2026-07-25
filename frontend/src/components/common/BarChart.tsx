// 縦軸ラベル付きの棒グラフ。
// 改善点資料「棒グラフの縦軸がなくて何のことか分からない」を解消するため、
// 左に目盛り(0〜最大値の間を4分割)を必ず表示する。

interface BarChartProps {
  data: { label: string; value: number }[]
  small?: boolean
}

export function BarChart({ data, small }: BarChartProps) {
  const max = Math.max(1, ...data.map((d) => d.value))
  const ticks = [4, 3, 2, 1, 0].map((i) => Math.round((max * i) / 4))

  return (
    <div className="bar-chart-wrap">
      <div className="bar-chart-axis" data-small={small ? 'true' : 'false'}>
        {ticks.map((tick, index) => (
          <span key={index}>{tick.toLocaleString('ja-JP')}</span>
        ))}
      </div>
      <div className={`bar-chart ${small ? 'small' : ''}`}>
        {data.length === 0 && <span className="muted">データがありません</span>}
        {data.map((item) => (
          <div key={item.label} title={`${item.label}: ${item.value}`}>
            <i style={{ height: `${(item.value / max) * 100}%` }} />
            <span>{item.label}</span>
          </div>
        ))}
      </div>
    </div>
  )
}
