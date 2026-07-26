// embedColorの入力コンポーネント。
// 設計書9.5節: カラーピッカーで選ぶと 0xRRGGBB 形式に自動変換する。16進数の手入力は必須にしない。
//
// Ver1改善点対応:
//   - 色を設定しても見た目に反映されていなかった問題 → プレビューのborder-leftに実際の色を反映する
//   - プレビューが横長で見にくかった問題 → 正方形に近い、コンパクトなプレビューにする
// Ver2改善点対応:
//   - 未設定に戻すボタンを追加
//   - 未設定時のデフォルト色を #87CEEB にする
//   - カラーピッカーの当たり判定が広すぎたため、プレビューの色見本(border-left)部分だけに絞る
//     (上部に独立したinput[type=color]を置かず、色見本自体をクリック可能なピッカーにする)

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
      <div
        className="embed-preview"
        style={{ borderLeftColor: cssColor, width: 220 }}
      >
        {/* 色見本(border-left)部分だけをカラーピッカーの当たり判定にする。
            ネイティブcolor inputを重ねて透明化し、見た目は色帯のクリックに見せる。 */}
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
          style={{ position: 'absolute', width: 0, height: 0, opacity: 0, pointerEvents: 'none' }}
        />
        <div className="embed-title">サンプル: XXXX(証券コード)決算速報</div>
        <div className="embed-body">プレビュー表示です</div>
      </div>
      <div className="color-picker-row" style={{ marginTop: 8 }}>
        <span className="badge">{value ?? `未設定(0x${DEFAULT_COLOR_HEX})`}</span>
        <button type="button" onClick={() => onChange(null)} disabled={value === null}>
          未設定に戻す
        </button>
      </div>
    </div>
  )
}
