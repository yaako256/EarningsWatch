// 一括インポート機能で使う、CSV/Excelファイルのパース処理。
// フロントエンド側でファイルをパースし、行データの配列にしてから API設計書 7章のインポートAPIへ送る想定。
//
// 想定する列見出し(1行目):
//   全体一括: ticker, companyName, groupName, notes, enabled
//   グループ単位: ticker, companyName, notes, enabled
// 大文字小文字・全角スペースの揺れをある程度吸収する。

import * as XLSX from 'xlsx'
import type { ImportRow } from '../types/api'

function normalizeHeader(header: string): string {
  return header.trim().toLowerCase().replace(/[\s_]/g, '')
}

const HEADER_ALIASES: Record<string, keyof ImportRow> = {
  ticker: 'ticker',
  companyname: 'companyName',
  会社名: 'companyName',
  企業名: 'companyName',
  groupname: 'groupName',
  グループ名: 'groupName',
  notes: 'notes',
  メモ: 'notes',
  enabled: 'enabled',
  状態: 'enabled',
}

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
