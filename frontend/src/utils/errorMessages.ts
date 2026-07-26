// エラーコード → 日本語メッセージの対応表(設計書15.1節)。
// ApiErrorが返すmessageは通信エラー時の汎用文言であることが多いため、
// フォームインラインエラー表示では基本的にこちらのコード起点のメッセージを優先して使う。

import type { ErrorCode } from '../types/api'

const MESSAGES: Record<ErrorCode, string> = {
  unauthorized: 'ログインの有効期限が切れました。再度ログインしてください。',
  forbidden: 'この操作を行う権限がありません。',
  not_found: '対象のデータが見つかりませんでした。一覧を更新します。',
  already_exists: 'すでに同じ内容のデータが存在します。',
  invalid_request: '入力内容に誤りがあります。項目を確認してください。',
  notify_config_missing: '通知先(Webhook URL等)が設定されていません。設定タブから登録してください。',
  notify_send_failed: '通知の送信に失敗しました。Webhook URLや接続状況を確認してください。',
  notify_rejected: '通知が送信先に拒否されました。設定内容を確認してください。',
  import_empty: '取り込める行がありませんでした。ファイルの内容を確認してください。',
  internal_error: 'サーバーでエラーが発生しました。時間をおいて再度お試しください。',
}

export function errorCodeMessage(code: string): string {
  return MESSAGES[code as ErrorCode] ?? '予期しないエラーが発生しました。'
}
