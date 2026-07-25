// 設計書10.2節・11.11節に基づく、一覧の読み込み中/空状態/エラー表示の共通部品。

interface EmptyStateProps {
  message: string
}

export function EmptyState({ message }: EmptyStateProps) {
  return (
    <div className="empty">
      <span>(´･ω･`) ｼｮﾎﾞｰﾝ</span>
      {message}
    </div>
  )
}

export function LoadingRow({ colSpan }: { colSpan: number }) {
  return (
    <tr>
      <td colSpan={colSpan} className="loading">
        <i /> 読み込み中...
      </td>
    </tr>
  )
}

export function ErrorState({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <div className="empty">
      <span>(´；ω；｀)</span>
      {message}
      {onRetry && <button onClick={onRetry}>再読み込み</button>}
    </div>
  )
}
