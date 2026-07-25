// 秘匿性の高い入力欄(Webhook URL等)。
// 設計書11.14節: type="password"は使わずマスク表示/解除トグルを自前で実装する
// (ブラウザのパスワードマネージャの誤反応を避けるため)。
//
// -webkit-text-security はFirefox非対応のため使わず、非表示時は編集不可の伏字表示にし、
// 編集したい場合は先に「表示する」で生値を出してから編集する方式にする。

import { useState } from 'react'

interface MaskedFieldProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
}

export function MaskedField({ value, onChange, placeholder }: MaskedFieldProps) {
  const [revealed, setRevealed] = useState(false)

  return (
    <div className="masked-field">
      <input
        type="text"
        value={revealed ? value : value ? '●'.repeat(Math.min(value.length, 24)) : ''}
        placeholder={placeholder}
        onChange={(e) => revealed && onChange(e.target.value)}
        readOnly={!revealed}
        className={revealed ? '' : 'masked-value'}
        autoComplete="off"
        data-lpignore="true"
      />
      <button
        type="button"
        className="masked-toggle"
        onClick={() => setRevealed((prev) => !prev)}
        aria-label={revealed ? '隠す' : '表示して編集する'}
        title={revealed ? '隠す' : '表示して編集する'}
      >
        {revealed ? '🙈' : '👁'}
      </button>
    </div>
  )
}
