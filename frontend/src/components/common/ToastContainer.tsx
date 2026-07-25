import { useToast } from '../../contexts/ToastContext'

export function ToastContainer() {
  const { toasts, dismissToast } = useToast()

  return (
    <div className="toasts">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast ${toast.kind === 'error' ? 'error' : ''}`}>
          <span>{toast.text}</span>
          <button onClick={() => dismissToast(toast.id)} aria-label="閉じる">
            ✕
          </button>
        </div>
      ))}
    </div>
  )
}
