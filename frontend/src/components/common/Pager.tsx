// AtCoder式ページネーション。
// 現在ページの近くは細かく(±2)、遠くは 3 4 5 6 7 8 10 15 25 50 ... のように間隔を粗くしながら
// 最初/最後のページには必ず到達できるようにする。

interface AtCoderPagerProps {
  currentPage: number
  totalPages: number
  onChange: (page: number) => void
}

function buildPageList(current: number, total: number): (number | 'ellipsis')[] {
  if (total <= 1) return [1]

  const near = new Set<number>()
  for (let offset = -2; offset <= 2; offset += 1) {
    const page = current + offset
    if (page >= 1 && page <= total) near.add(page)
  }
  near.add(1)
  near.add(total)

  // 遠方は倍々に増える間隔で少数のランドマークだけ提示する。
  let step = 3
  let page = current
  while (page < total) {
    page += step
    if (page < total) near.add(page)
    step = Math.min(step * 2, 25)
  }
  step = 3
  page = current
  while (page > 1) {
    page -= step
    if (page > 1) near.add(page)
    step = Math.min(step * 2, 25)
  }

  const sorted = [...near].sort((a, b) => a - b)
  const result: (number | 'ellipsis')[] = []
  sorted.forEach((p, index) => {
    if (index > 0 && p - sorted[index - 1] > 1) {
      result.push('ellipsis')
    }
    result.push(p)
  })
  return result
}

export function AtCoderPager({ currentPage, totalPages, onChange }: AtCoderPagerProps) {
  const pages = buildPageList(currentPage, totalPages)

  return (
    <div className="pager">
      <button disabled={currentPage <= 1} onClick={() => onChange(currentPage - 1)}>
        ← 前へ
      </button>
      {pages.map((page, index) =>
        page === 'ellipsis' ? (
          <span key={`ellipsis-${index}`} className="muted">
            …
          </span>
        ) : (
          <button
            key={page}
            className={page === currentPage ? 'active' : ''}
            onClick={() => onChange(page)}
          >
            {page}
          </button>
        ),
      )}
      <button disabled={currentPage >= totalPages} onClick={() => onChange(currentPage + 1)}>
        次へ →
      </button>
    </div>
  )
}

export function SimplePager({
  currentPage,
  hasNext,
  onChange,
}: {
  currentPage: number
  hasNext: boolean
  onChange: (page: number) => void
}) {
  return (
    <div className="pager">
      <button disabled={currentPage <= 1} onClick={() => onChange(currentPage - 1)}>
        ← 前へ
      </button>
      <span className="badge">{currentPage} ページ目</span>
      <button disabled={!hasNext} onClick={() => onChange(currentPage + 1)}>
        次へ →
      </button>
    </div>
  )
}
