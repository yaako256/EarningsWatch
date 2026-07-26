// フィルタの一括インポートモーダル。全体版(グループ一覧画面)・グループ単位版(フィルタタブ)で共有する。
// 設計書7.4節: ボタン押下→モーダル→ファイル選択/ドラッグ&ドロップで実行する。
//
// Ver2改善点対応: ネイティブの<input type="file">は「選択されていません」というテキスト部分も
// クリック領域に含んでしまうため非表示にし、代わりに独自の「ファイルを選択」ボタンだけを
// クリック可能にしてinput.click()を呼ぶ方式にする。
// Ver4改善点対応:
//   - 警告・エラーメッセージが読点区切りの1行になっていて読みにくかったため、行ごとに改行して表示する
//   - 必須列欠落等の致命的エラー(errorRows)がAPIから返っていても、以前は警告(warnings)と
//     見分けがつきにくかったため、見出しを分けて明確に区別する
//   - ファイル選択時点のエラー(拡張子違い・列名不一致・破損ファイル)を種別ごとに出し分ける

import { useRef, useState } from 'react'
import type { DragEvent } from 'react'
import { Modal } from './Modal'
import { parseImportFile, IMPORT_COLUMN_NAMES, ImportFileError } from '../../utils/parseImportFile'
import type { ImportResult, ImportRow } from '../../types/api'
import { useToast } from '../../contexts/ToastContext'

interface ImportModalProps {
  title: string
  scopeHint: string
  onImport: (rows: ImportRow[], dryRun: boolean) => Promise<ImportResult>
  onClose: () => void
  onImported: () => void
}

export function ImportModal({ title, scopeHint, onImport, onClose, onImported }: ImportModalProps) {
  const { showToast } = useToast()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [rows, setRows] = useState<ImportRow[] | undefined>(undefined)
  const [fileName, setFileName] = useState('')
  const [isDragOver, setIsDragOver] = useState(false)
  const [result, setResult] = useState<ImportResult | undefined>(undefined)
  const [fileError, setFileError] = useState('')
  const [submitError, setSubmitError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleFile = async (file: File) => {
    setFileError('')
    setSubmitError('')
    setResult(undefined)
    try {
      const parsed = await parseImportFile(file)
      setRows(parsed)
      setFileName(file.name)
    } catch (err) {
      setRows(undefined)
      setFileName('')
      if (err instanceof ImportFileError) {
        setFileError(err.message)
      } else {
        setFileError('ファイルを読み取れませんでした。CSVまたはExcel形式のファイルを選択してください。')
      }
    }
  }

  const handleDrop = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    setIsDragOver(false)
    const file = e.dataTransfer.files[0]
    if (file) void handleFile(file)
  }

  const runImport = async (dryRun: boolean) => {
    if (!rows) return
    setIsSubmitting(true)
    setSubmitError('')
    try {
      const outcome = await onImport(rows, dryRun)
      setResult(outcome)
      if (!dryRun) {
        showToast('success', `${outcome.importedCount}件のフィルタを取り込みました。`)
        onImported()
      }
    } catch {
      setSubmitError('取り込みに失敗しました。ファイル内容を確認するか、時間をおいて再度お試しください。')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Modal title={title} onClose={onClose} wide>
      <p style={{ color: 'var(--text-muted)' }}>{scopeHint}</p>
      <p style={{ color: 'var(--text-muted)', fontSize: 13 }}>
        列見出し: {Object.values(IMPORT_COLUMN_NAMES).join(' / ')}
      </p>
      <div
        onDragOver={(e) => {
          e.preventDefault()
          setIsDragOver(true)
        }}
        onDragLeave={() => setIsDragOver(false)}
        onDrop={handleDrop}
        style={{
          border: `2px dashed ${isDragOver ? 'var(--accent)' : 'var(--border)'}`,
          borderRadius: 10,
          padding: 32,
          textAlign: 'center',
          color: 'var(--text-muted)',
          marginBottom: 16,
        }}
      >
        {fileName ? (
          <span>選択中のファイル: {fileName}({rows?.length ?? 0}行)</span>
        ) : (
          <span>ここにCSV/Excelファイルをドラッグ&ドロップ</span>
        )}
        <div style={{ marginTop: 12 }}>
          {/* ネイティブinputは非表示にし、ボタンのみをクリック可能領域にする */}
          <input
            ref={fileInputRef}
            type="file"
            accept=".csv,.xlsx,.xls"
            style={{ display: 'none' }}
            onChange={(e) => {
              const file = e.target.files?.[0]
              if (file) void handleFile(file)
            }}
          />
          <button type="button" onClick={() => fileInputRef.current?.click()}>
            ファイルを選択
          </button>
        </div>
      </div>

      {fileError && <p className="inline-error">{fileError}</p>}
      {submitError && <p className="inline-error">{submitError}</p>}

      {result && (
        <div className="notice-banner" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
          <div>
            取り込み対象: {result.importedCount}件 / スキップ(空行): {result.skippedEmptyRows}件 / 重複: {result.duplicateCount}件
          </div>
          {result.createdGroups.length > 0 && (
            <div>新規作成されたグループ: {result.createdGroups.map((g) => g.name).join('、')}</div>
          )}
          {result.pausedGroups.length > 0 && (
            <div>一時停止中のグループ: {result.pausedGroups.map((g) => g.name).join('、')}</div>
          )}

          {/* Ver4改善点対応: 必須列欠落等の致命的エラー(errorRows)は、警告(warnings)とは
              見出しを分けたうえで、1行1件の縦並びリストとして表示する(読点連結による
              読みにくさを解消する)。 */}
          {result.errorRows.length > 0 && (
            <div>
              <strong style={{ color: 'var(--danger)' }}>エラーのある行(取り込まれませんでした)</strong>
              <ul style={{ margin: '4px 0 0', paddingLeft: 20 }}>
                {result.errorRows.map((r) => (
                  <li key={r.rowNumber}>
                    {r.rowNumber}行目: {r.reason}
                  </li>
                ))}
              </ul>
            </div>
          )}
          {result.warnings.length > 0 && (
            <div>
              <strong style={{ color: 'var(--warning)' }}>警告(取り込みは行われました)</strong>
              <ul style={{ margin: '4px 0 0', paddingLeft: 20 }}>
                {result.warnings.map((w) => (
                  <li key={w.rowNumber}>
                    {w.rowNumber}行目: {w.message}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      <div className="page-header-actions" style={{ justifyContent: 'flex-end', marginTop: 16 }}>
        <button onClick={onClose}>閉じる</button>
        <button disabled={!rows || isSubmitting} onClick={() => runImport(true)}>
          内容を確認する(試験実行)
        </button>
        <button className="primary" disabled={!rows || isSubmitting} onClick={() => runImport(false)}>
          {isSubmitting ? '取り込み中...' : '取り込む'}
        </button>
      </div>
    </Modal>
  )
}
