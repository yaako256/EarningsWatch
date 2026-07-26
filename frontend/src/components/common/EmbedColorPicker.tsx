// embedColorの入力コンポーネント。
// 設計書9.5節: カラーピッカーで選ぶと 0xRRGGBB 形式に自動変換する。16進数の手入力は必須にしない。
//
// Ver1改善点対応:
//   - 色を設定しても見た目に反映されていなかった問題 → プレビューのborder-leftに実際の色を反映する
//   - プレビューが横長で見にくかった問題 → 正方形に近い、コンパクトなプレビューにする
// Ver2改善点対応:
//   - 未設定に戻すボタンを追加
//   - 未設定時のデフォルト色を #87CEEB にする
// Ver3改善点対応:
//   - 色見本を絶対配置でプレビューに重ねる実装だと、当たり判定がプレビュー全体ににじんで
//     見えてしまっていたため、絶対配置をやめて独立した小さな正方形ボタンとして
//     プレビューの外(隣)に配置する。当たり判定とクリック可能に見える範囲が完全に一致する。

import { useRef } from 'react'

interface EmbedColorPickerProps {
  value: string | null // "0xRRGGBB" 形式
  onChange: (value: string | null) => void
}

const DEFAULT_COLOR_HEX = '87CEEB'

function toCssColor(embedColor: string | null): string {
  const hex = embedColor ? embedColor.replace(/^0x/i, '') : DEFAULT_COLOR_HEX
  return `#${hex.padStart(6, '0')}`
}

function toEmbedColor(cssColor: string): string {
  return `0x${cssColor.replace('#', '').toUpperCase()}`
}

export function EmbedColorPicker({ value, onChange }: EmbedColorPickerProps) {
  const cssColor = toCssColor(value)
  const colorInputRef = useRef<HTMLInputElement>(null)

  return (
    <div>
      <div className="embed-color-row">
        <button
          type="button"
          className="embed-color-swatch"
          style={{ background: cssColor }}
          onClick={() => colorInputRef.current?.click()}
          aria-label="Embed色を変更"
          title="クリックして色を変更"
        />
        <input
          ref={colorInputRef}
          type="color"
          value={cssColor}
          onChange={(e) => onChange(toEmbedColor(e.target.value))}
          aria-label="Embed色"
          tabIndex={-1}
        />
        <span className="badge">{value ?? `未設定(0x${DEFAULT_COLOR_HEX})`}</span>
        <button type="button" onClick={() => onChange(null)} disabled={value === null}>
          未設定に戻す
        </button>
      </div>
      <div className="embed-preview" style={{ borderLeftColor: cssColor, width: 220 }}>
        <div className="embed-title">サンプル: XXXX(証券コード)決算速報</div>
        <div className="embed-body">プレビュー表示です</div>
      </div>
    </div>
  )
}
