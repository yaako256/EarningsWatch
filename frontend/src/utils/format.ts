// 日時・数値の表示フォーマットを共通化する。
// フォーマット規則は フロントエンド設計書.md 11.3節/11.4節に準拠する。
//   - 日時は yyyy/MM/dd HH:mm (秒が必要な場合は yyyy/MM/dd HH:mm:ss)
//   - 表示はJSTに変換するが、画面上に「JST」等の注記は付けない
//   - 数値は3桁区切りのカンマ区切り
//   - 空データは "-" で統一する

function pad(value: number, length = 2): string {
  return String(value).padStart(length, '0')
}

/** APIが返すUTCのISO8601文字列をJSTの yyyy/MM/dd HH:mm[:ss] 形式に変換する。 */
export function formatDateTime(value?: string | null, withSeconds = false): string {
  if (!value) return '-'
  const date = new Date(value)
  const jstFormatter = new Intl.DateTimeFormat('en-US', {
    timeZone: 'Asia/Tokyo',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
  const parts = Object.fromEntries(jstFormatter.formatToParts(date).map((p) => [p.type, p.value]))
  const base = `${parts.year}/${parts.month}/${parts.day} ${parts.hour}:${parts.minute}`
  return withSeconds ? `${base}:${parts.second}` : base
}

/** 「stopped_at」のように「いつからか」が伝わりにくい単発の日時表示に、開始を明示する接頭辞を付ける。 */
export function formatSince(value?: string | null): string {
  if (!value) return '-'
  return `${formatDateTime(value)}〜`
}

/** 3桁区切りのカンマを入れる。null/undefinedは "-"。 */
export function formatNumber(value?: number | null): string {
  if (value == null) return '-'
  return new Intl.NumberFormat('ja-JP').format(value)
}

/** 通知成功率(0〜1の小数)をパーセント表記にする。実績なしの場合は専用文言。 */
export function formatRate(value: number | null): string {
  if (value == null) return '実績なし'
  return `${(value * 100).toFixed(1)}%`
}

/**
 * エクスポートファイル名の先頭に、実行日時(JST, yyyymmddhhmm)を付与してユニーク化する。
 * 例: exportFileName('filters.xlsx') -> '202607251530_filters.xlsx'
 */
export function exportFileName(baseName: string): string {
  const now = new Date()
  const jstFormatter = new Intl.DateTimeFormat('en-US', {
    timeZone: 'Asia/Tokyo',
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
  const parts = Object.fromEntries(jstFormatter.formatToParts(now).map((p) => [p.type, p.value]))
  const prefix = `${parts.year}${parts.month}${parts.day}${parts.hour}${parts.minute}`
  return `${prefix}_${baseName}`
}

/** チェックボックス等が生成する簡易一意ID。React keyや紐付け用。 */
export function uniqueSuffix(): string {
  return Math.random().toString(36).slice(2, 8)
}

// padは将来の桁揃え用途(ログのミリ秒表示等)に備えてexportしておく。
export { pad }
