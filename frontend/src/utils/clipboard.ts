// クリップボードへのコピー処理。
//
// Ver2改善点対応: navigator.clipboard は HTTPS または localhost (セキュアコンテキスト) でしか
// 使用できず、IPアドレス直打ちのHTTP環境等では navigator.clipboard 自体が存在しないか、
// 呼び出しが失敗する。以前の実装はこの失敗を無視していたため、実際にはコピーされていないのに
// 「コピーしました」という成功トーストが出てしまっていた。
//
// ここでは navigator.clipboard が使えない場合、非表示のtextarea+document.execCommand('copy')に
// フォールアックする。どちらも失敗した場合は例外を投げ、呼び出し側で失敗を通知できるようにする。

export async function copyToClipboard(text: string): Promise<void> {
  if (navigator.clipboard && window.isSecureContext) {
    await navigator.clipboard.writeText(text)
    return
  }

  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.opacity = '0'
  document.body.appendChild(textarea)
  textarea.focus()
  textarea.select()
  try {
    const succeeded = document.execCommand('copy')
    if (!succeeded) {
      throw new Error('execCommand(copy) failed')
    }
  } finally {
    document.body.removeChild(textarea)
  }
}
