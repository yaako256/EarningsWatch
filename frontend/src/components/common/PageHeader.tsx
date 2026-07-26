// ページ見出し共通コンポーネント。改善点まとめ資料「タイトル左にアイコンが欲しい」に対応。

import type { ReactNode } from 'react'

interface PageHeaderProps {
  icon: string
  title: string
  actions?: ReactNode
}

export function PageHeader({ icon, title, actions }: PageHeaderProps) {
  return (
    <div className="page-header">
      <h1>
        <span className="page-icon">{icon}</span>
        {title}
      </h1>
      {actions && <div className="page-header-actions">{actions}</div>}
    </div>
  )
}
