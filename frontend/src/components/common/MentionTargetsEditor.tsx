// mentionTargetsの編集UI。
// 設計書9.5節: フリーテキスト入力ではなく、種類ごとの選択式UIで組み立てる。
// API設計書 6章記載の形式: "user:<ID>" / "role:<ID>" / "everyone" / "here"
// (タイムスタンプ表示は本文側の機能のため、ここでは対象外。mentionTargetsの構成要素のみ扱う)

import { useState } from 'react'

type TargetKind = 'user' | 'role' | 'everyone' | 'here'

interface MentionTargetsEditorProps {
  targets: string[]
  onChange: (targets: string[]) => void
}

function describeTarget(target: string): string {
  if (target === 'everyone') return '@everyone'
  if (target === 'here') return '@here'
  if (target.startsWith('user:')) return `ユーザー: ${target.slice(5)}`
  if (target.startsWith('role:')) return `ロール: ${target.slice(5)}`
  return target
}

export function MentionTargetsEditor({ targets, onChange }: MentionTargetsEditorProps) {
  const [kind, setKind] = useState<TargetKind>('user')
  const [idValue, setIdValue] = useState('')

  const addTarget = () => {
    const value =
      kind === 'everyone' ? 'everyone' : kind === 'here' ? 'here' : `${kind}:${idValue.trim()}`
    if ((kind === 'user' || kind === 'role') && idValue.trim() === '') return
    if (targets.includes(value)) return
    onChange([...targets, value])
    setIdValue('')
  }

  const removeTarget = (target: string) => {
    onChange(targets.filter((t) => t !== target))
  }

  return (
    <div className="mentions">
      <ul>
        {targets.length === 0 && <li className="muted">メンション対象は未設定です</li>}
        {targets.map((target) => (
          <li key={target}>
            {describeTarget(target)}
            <button type="button" onClick={() => removeTarget(target)} aria-label="削除">
              ✕
            </button>
          </li>
        ))}
      </ul>
      <div className="mention-adder">
        <select value={kind} onChange={(e) => setKind(e.target.value as TargetKind)}>
          <option value="user">ユーザーID指定</option>
          <option value="role">ロールID指定</option>
          <option value="everyone">@everyone</option>
          <option value="here">@here</option>
        </select>
        {(kind === 'user' || kind === 'role') && (
          <input
            placeholder={kind === 'user' ? 'Discord ユーザーID' : 'Discord ロールID'}
            value={idValue}
            onChange={(e) => setIdValue(e.target.value)}
          />
        )}
        <button type="button" onClick={addTarget}>
          + 追加
        </button>
      </div>
    </div>
  )
}
