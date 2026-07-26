// スマホ幅(768px未満)でのアクセス時に警告を出すコンポーネント。
// Ver4改善点対応: 以前は実装されていなかった機能。
//
// 「それでも開きますか？」の選択を提供し、"はい"を選んだ場合はそのセッション中は再確認しない
// (sessionStorageではなくReact stateで保持。ページ再読み込みでは再度警告が出る仕様とする。
//  常時許可させる必要が出た場合はsessionStorageへの切り替えを検討する)。

import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'

const MOBILE_BREAKPOINT_PX = 768

function isNarrowScreen(): boolean {
  return window.innerWidth < MOBILE_BREAKPOINT_PX
}

export function MobileGate({ children }: { children: ReactNode }) {
  const [isNarrow, setIsNarrow] = useState(isNarrowScreen)
  const [dismissed, setDismissed] = useState(false)

  useEffect(() => {
    const handleResize = () => setIsNarrow(isNarrowScreen())
    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  const showGate = isNarrow && !dismissed

  // html/bodyのmin-width: 1080px(デスクトップ向けレイアウト制約)を、
  // この警告画面を表示している間だけ打ち消す。
  useEffect(() => {
    document.documentElement.classList.toggle('mobile-gate-active', showGate)
    return () => document.documentElement.classList.remove('mobile-gate-active')
  }, [showGate])

  if (showGate) {
    return (
      <div className="mobile-gate">
        <div className="mobile-gate-card">
          <div className="mobile-gate-icon">📵</div>
          <h1>スマホには対応していません</h1>
          <p>
            この画面はスマートフォンなどの狭い画面幅には最適化されていません。
            PCまたはタブレットでのご利用を推奨します。
          </p>
          <p>それでも開きますか？</p>
          <button className="primary" onClick={() => setDismissed(true)}>
            それでも開く
          </button>
        </div>
      </div>
    )
  }

  return <>{children}</>
}
