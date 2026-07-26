import { Navigate, Outlet, useLocation } from 'react-router-dom'
import { useAuth } from '../contexts/AuthContext'

export function AuthLoading() {
  return (
    <div className="auth-loading">
      <span className="loading">
        <i /> 読み込み中...
      </span>
    </div>
  )
}

export function RequireAuth() {
  const { user, isLoading } = useAuth()
  const location = useLocation()

  if (isLoading) return <AuthLoading />
  if (!user) return <Navigate to="/login" state={{ from: location }} replace />
  return <Outlet />
}

export function RequireAdmin() {
  const { user } = useAuth()
  if (user?.role !== 'admin') return <Navigate to="/forbidden" replace />
  return <Outlet />
}
