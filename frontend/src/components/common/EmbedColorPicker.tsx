// embedColorの入力コンポーネント。
// 設計書9.5節: カラーピッカーで選ぶと 0xRRGGBB 形式に自動変換する。16進数の手入力は必須にしない。
//
// 改善点まとめ資料の指摘に対応:
//   - 色を設定しても見た目に反映されていなかった問題 → プレビューのborder-leftに実際の色を反映する
//   - プレビューが横長で見にくかった問題 → 正方形に近い、コンパクトなプレビューにする

interface EmbedColorPickerProps {
  value: string | null // "0xRRGGBB" 形式
  onChange: (value: string) => void
}

function toCssColor(embedColor: string | null): string {
  if (!embedColor) return '#8a8a8a'
  const hex = embedColor.replace(/^0x/i, '')
  return `#${hex.padStart(6, '0')}`
}

function toEmbedColor(cssColor: string): string {
  return `0x${cssColor.replace('#', '').toUpperCase()}`
}

export function EmbedColorPicker({ value, onChange }: EmbedColorPickerProps) {
  const cssColor = toCssColor(value)

  return (
    <div>
      <div className="color-picker-row">
        <input
          type="color"
          value={cssColor}
          onChange={(e) => onChange(toEmbedColor(e.target.value))}
          aria-label="Embed色"
        />
        <span className="badge">{value ?? '未設定'}</span>
      </div>
      <div
        className="embed-preview"
        style={{ borderLeftColor: cssColor, width: 220 }}
      >
        <div className="embed-title">サンプル: XXXX(証券コード)決算速報</div>
        <div className="embed-body">プレビュー表示です</div>
      </div>
    </div>
  )
}
