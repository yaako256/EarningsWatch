// mentionTargetsの編集UI。
// 設計書9.5節: フリーテキスト入力ではなく、種類ごとの選択式UIで組み立てる。
// API設計書 6章記載の形式: "user:<ID>" / "role:<ID>" / "everyone" / "here" / "time:<スタイル文字>"
//
// Ver2改善点対応:
//   - 時間メンション(Discordのタイムスタンプ表示 <t:unix:style>)が選択できなかったため追加する。
//     スタイル文字はAPI設計書記載の t/T/d/D/f/F/R のいずれか。
//   - 「+ 追加」ボタンが縦長・折り返しされてしまっていたため、常に横一列に収まるレイアウトにする。

import { useState } from 'react'

type TargetKind = 'user' | 'role' | 'everyone' | 'here' | 'time'

const TIME_STYLES: { value: string; label: string }[] = [
  { value: 't', label: '短い時刻 (t)' },
  { value: 'T', label: '長い時刻 (T)' },
  { value: 'd', label: '短い日付 (d)' },
  { value: 'D', label: '長い日付 (D)' },
  { value: 'f', label: '日付+時刻 (f)' },
  { value: 'F', label: '曜日+日付+時刻 (F)' },
  { value: 'R', label: '相対時間 (R)' },
]

interface MentionTargetsEditorProps {
  targets: string[]
  onChange: (targets: string[]) => void
}

function describeTarget(target: string): string {
  if (target === 'everyone') return '@everyone'
  if (target === 'here') return '@here'
  if (target.startsWith('user:')) return `ユーザー: ${target.slice(5)}`
  if (target.startsWith('role:')) return `ロール: ${target.slice(5)}`
  if (target.startsWith('time:')) {
    const style = target.slice(5)
    return `時間表示: ${TIME_STYLES.find((s) => s.value === style)?.label ?? style}`
  }
  return target
}

export function MentionTargetsEditor({ targets, onChange }: MentionTargetsEditorProps) {
  const [kind, setKind] = useState<TargetKind>('user')
  const [idValue, setIdValue] = useState('')
  const [timeStyle, setTimeStyle] = useState('R')

  const addTarget = () => {
    let value: string
    if (kind === 'everyone' || kind === 'here') {
      value = kind
    } else if (kind === 'time') {
      value = `time:${timeStyle}`
    } else {
      if (idValue.trim() === '') return
      value = `${kind}:${idValue.trim()}`
    }
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
          <option value="time">時間表示</option>
        </select>
        {(kind === 'user' || kind === 'role') && (
          <input
            placeholder={kind === 'user' ? 'Discord ユーザーID' : 'Discord ロールID'}
            value={idValue}
            onChange={(e) => setIdValue(e.target.value)}
          />
        )}
        {kind === 'time' && (
          <select value={timeStyle} onChange={(e) => setTimeStyle(e.target.value)}>
            {TIME_STYLES.map((s) => (
              <option key={s.value} value={s.value}>
                {s.label}
              </option>
            ))}
          </select>
        )}
        <button type="button" onClick={addTarget}>
          + 追加
        </button>
      </div>
    </div>
  )
}
