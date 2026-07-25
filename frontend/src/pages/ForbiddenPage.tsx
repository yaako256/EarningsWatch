import { Link } from 'react-router-dom'

export function ForbiddenPage() {
  return (
    <div className="forbidden">
      <div>
        <h1>🚫 権限がありません</h1>
        <p style={{ color: 'var(--text-muted)', marginBottom: 20 }}>
          この画面を表示する権限がありません。管理者にお問い合わせください。
        </p>
        <Link to="/dashboard" className="link">
          ダッシュボードに戻る
        </Link>
      </div>
    </div>
  )
}
