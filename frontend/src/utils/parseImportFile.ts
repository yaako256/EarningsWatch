// 一括インポート機能で使う、CSV/Excelファイルのパース処理。
// フロントエンド側でファイルをパースし、行データの配列にしてから API設計書 7章のインポートAPIへ送る想定。
//
// Ver2改善点対応: 列見出しは他システムのCSVと混同しないよう、
// "EarningsWatch_" プレフィックス付きの固定名のみを受け付ける(緩いエイリアスは廃止)。
//   全体一括:     EarningsWatch_Ticker, EarningsWatch_CompanyName, EarningsWatch_GroupName,
//                 EarningsWatch_Notes, EarningsWatch_Enabled
//   グループ単位: EarningsWatch_Ticker, EarningsWatch_CompanyName,
//                 EarningsWatch_Notes, EarningsWatch_Enabled
// 大文字小文字・前後の空白の揺れのみ吸収する(列名自体の意訳・別名は受け付けない)。

import * as XLSX from 'xlsx'
import type { ImportRow } from '../types/api'

function normalizeHeader(header: string): string {
  return header.trim().toLowerCase()
}

const HEADER_ALIASES: Record<string, keyof ImportRow> = {
  earningswatch_ticker: 'ticker',
  earningswatch_companyname: 'companyName',
  earningswatch_groupname: 'groupName',
  earningswatch_notes: 'notes',
  earningswatch_enabled: 'enabled',
}

export const IMPORT_COLUMN_NAMES = {
  ticker: 'EarningsWatch_Ticker',
  companyName: 'EarningsWatch_CompanyName',
  groupName: 'EarningsWatch_GroupName',
  notes: 'EarningsWatch_Notes',
  enabled: 'EarningsWatch_Enabled',
} as const

function rowsFromSheetJson(json: unknown[][]): ImportRow[] {
  if (json.length === 0) return []
  const [headerRow, ...dataRows] = json
  const columns = headerRow.map((h) => HEADER_ALIASES[normalizeHeader(String(h ?? ''))])

  return dataRows
    .filter((row) => row.some((cell) => cell !== undefined && cell !== ''))
    .map((row) => {
      const record: Partial<ImportRow> = {}
      columns.forEach((key, index) => {
        if (!key) return
        const raw = row[index]
        if (raw === undefined || raw === '') return
        if (key === 'enabled') {
          const text = String(raw).trim().toLowerCase()
          record.enabled = ['true', '1', '有効', 'yes'].includes(text)
        } else {
          ;(record as Record<string, unknown>)[key] = String(raw)
        }
      })
      return record as ImportRow
    })
}

/** .csv または .xlsx/.xls ファイルを読み取り、ImportRow[] に変換する。 */
export async function parseImportFile(file: File): Promise<ImportRow[]> {
  const buffer = await file.arrayBuffer()
  const workbook = XLSX.read(buffer, { type: 'array' })
  const firstSheetName = workbook.SheetNames[0]
  const sheet = workbook.Sheets[firstSheetName]
  const json = XLSX.utils.sheet_to_json<unknown[]>(sheet, { header: 1, raw: false })
  return rowsFromSheetJson(json as unknown[][])
}
