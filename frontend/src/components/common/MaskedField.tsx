// 秘匿性の高い入力欄(Webhook URL等)。
//
// Ver4改善点対応: 以前は非表示状態だと readOnly にしており編集自体ができなかった。
// 「パスワード入力欄のように、隠したまま編集できるようにしたい」との指摘のため、
// 非表示時は type="password" (ブラウザ標準の伏字表示、入力は可能)、
// 表示時は type="text" に切り替える方式にする。
// (type="password"はブラウザのパスワードマネージャの保存提案を誘発する場合があるが、
//  autoComplete="new-password" 指定でこれを抑止する。)

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
        type={revealed ? 'text' : 'password'}
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        autoComplete="new-password"
        data-lpignore="true"
      />
      <button
        type="button"
        className="masked-toggle"
        onClick={() => setRevealed((prev) => !prev)}
        aria-label={revealed ? '隠す' : '表示する'}
        title={revealed ? '隠す' : '表示する'}
      >
        {revealed ? '🙈' : '👁'}
      </button>
    </div>
  )
}
