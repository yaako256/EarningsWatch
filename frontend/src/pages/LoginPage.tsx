import { useState } from 'react'
import type { FormEvent } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuth } from '../contexts/AuthContext'
import { ApiError } from '../api/client'

interface LocationState {
  from?: { pathname: string }
}

export function LoginPage() {
  const { user, login } = useAuth()
  const location = useLocation()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  // Ver2改善点対応:
  //   - ログインページから直接ログインした場合はダッシュボードへ遷移する
  //   - RequireAuthに弾かれてログインページに飛ばされた場合(location.stateにfromが入っている)は、
  //     元々開こうとしていたページへ戻す
  //   - リフレッシュ(セッションが生きたままのページ再読み込み)はRequireAuth側でisLoading中に
  //     ログインページへ飛ばさないため、このページ自体は影響を受けない
  if (user) {
    const state = location.state as LocationState | null
    const from = state?.from?.pathname ?? '/dashboard'
    return <Navigate to={from} replace />
  }

  // Ver4改善点対応: ApiErrorのmessageは共通のコード変換表(errorCodeMessage)由来のため、
  // unauthorizedが「ログインの有効期限切れ」という文言になっており、ログイン失敗時に
  // 出すには不適切だった。ログイン画面ではAPI設計書4.2節記載の2種類のエラーコード
  // (unauthorized: ユーザ名/パスワード不一致、forbidden: アカウント無効化)に応じて
  // 専用の文言に差し替える。
  const loginErrorMessage = (err: unknown): string => {
    if (err instanceof ApiError) {
      if (err.code === 'unauthorized') return 'ユーザ名またはパスワードが違っています。'
      if (err.code === 'forbidden') return 'あなたはBANされています。管理者にお問い合わせください。'
      return err.message
    }
    return 'ログインに失敗しました。'
  }

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError('')
    setIsSubmitting(true)
    try {
      await login(username, password)
    } catch (err) {
      setError(loginErrorMessage(err))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="login">
      <form onSubmit={handleSubmit}>
        <div className="brand">
          <span>📡</span>
          EarningsWatch
        </div>
        <p className="tagline">決算速報の通知設定を管理します</p>

        <label>
          ユーザー名
          <input value={username} onChange={(e) => setUsername(e.target.value)} autoFocus />
        </label>
        <label>
          パスワード
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </label>

        {error && <p className="inline-error">{error}</p>}

        <button className="primary" type="submit" disabled={isSubmitting}>
          {isSubmitting ? 'ログイン中...' : 'ログイン'}
        </button>
      </form>
    </div>
  )
}
