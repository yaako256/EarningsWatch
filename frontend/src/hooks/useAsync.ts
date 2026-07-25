// 一覧・詳細データ取得を共通化するカスタムフック。
// loading中はテーブルヘッダーを先に表示しておき、データ部分だけスピナーにする使い方を想定(設計書10.2節)。

import { useCallback, useEffect, useState } from 'react'
import { ApiError } from '../api/client'

interface AsyncState<T> {
  data: T | undefined
  isLoading: boolean
  error: string | undefined
}

function errorMessage(error: unknown): string {
  if (error instanceof ApiError) return error.message
  return '通信に失敗しました。時間をおいて再度お試しください。'
}

export function useAsync<T>(load: () => Promise<T>, deps: unknown[] = []) {
  const [state, setState] = useState<AsyncState<T>>({ data: undefined, isLoading: true, error: undefined })

  const reload = useCallback(() => {
    setState((prev) => ({ ...prev, isLoading: true, error: undefined }))
    load()
      .then((data) => setState({ data, isLoading: false, error: undefined }))
      .catch((error) => setState({ data: undefined, isLoading: false, error: errorMessage(error) }))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps)

  useEffect(() => {
    reload()
  }, [reload])

  return { ...state, reload }
}

export { errorMessage }
