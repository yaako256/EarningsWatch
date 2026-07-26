// お知らせ板本文用の、必要最小限のMarkdown→HTML変換。
// 外部ライブラリを追加しない方針(設計書2章)のため、見出し・強調・リスト・リンク・改行のみを素朴に変換する。
// contentMarkdownはユーザ(管理者)自身が書く前提のコンテンツだが、念のためHTMLエスケープしてからタグを組み立てる。

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

function renderInline(text: string): string {
  let html = escapeHtml(text)
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>')
  html = html.replace(/`([^`]+?)`/g, '<code>$1</code>')
  html = html.replace(/\[(.+?)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>')
  return html
}

export function renderMarkdown(markdown: string): string {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n')
  const htmlParts: string[] = []
  let listBuffer: string[] = []

  const flushList = () => {
    if (listBuffer.length === 0) return
    htmlParts.push(`<ul>${listBuffer.map((item) => `<li>${renderInline(item)}</li>`).join('')}</ul>`)
    listBuffer = []
  }

  for (const line of lines) {
    const heading = line.match(/^(#{1,6})\s+(.*)$/)
    const listItem = line.match(/^[-*]\s+(.*)$/)

    if (heading) {
      flushList()
      const level = heading[1].length
      htmlParts.push(`<h${level}>${renderInline(heading[2])}</h${level}>`)
    } else if (listItem) {
      listBuffer.push(listItem[1])
    } else if (line.trim() === '') {
      flushList()
    } else {
      flushList()
      htmlParts.push(`<p>${renderInline(line)}</p>`)
    }
  }
  flushList()
  return htmlParts.join('\n')
}
