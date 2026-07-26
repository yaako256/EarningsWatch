// 一括インポート機能で使う、CSV/Excelファイルのパース処理。
// フロントエンド側でファイルをパースし、行データの配列にしてから API設計書 7章のインポートAPIへ送る想定。
//
// Ver2改善点対応: 列見出しは他システムのCSVと混同しないよう、固定プレフィックス付きの固定名のみを受け付ける。
// Ver4改善点対応: プレフィックスが長い("EarningsWatch_")との指摘のため "EW_" に短縮。
//   全体一括:     EW_Ticker, EW_CompanyName, EW_GroupName, EW_Notes, EW_Enabled
//   グループ単位: EW_Ticker, EW_CompanyName, EW_Notes, EW_Enabled
// 大文字小文字・前後の空白の揺れのみ吸収する(列名自体の意訳・別名は受け付けない)。
//
// Ver4改善点対応: 以前は「拡張子が違う」「列名が想定と違う」をすべて一律の
// 「読み取れませんでした」に丸めていたため、原因の切り分けができなかった。
// ImportFileError で失敗理由を区別できるようにする。

import * as XLSX from 'xlsx'
import type { ImportRow } from '../types/api'

export type ImportFileErrorReason = 'unsupported_extension' | 'unreadable_file' | 'columns_not_found'

export class ImportFileError extends Error {
  reason: ImportFileErrorReason
  constructor(reason: ImportFileErrorReason, message: string) {
    super(message)
    this.reason = reason
  }
}

const SUPPORTED_EXTENSIONS = ['.csv', '.xlsx', '.xls']

function normalizeHeader(header: string): string {
  return header.trim().toLowerCase()
}

const HEADER_ALIASES: Record<string, keyof ImportRow> = {
  ew_ticker: 'ticker',
  ew_companyname: 'companyName',
  ew_groupname: 'groupName',
  ew_notes: 'notes',
  ew_enabled: 'enabled',
  ew_grouppausedat: 'groupPausedAt',
}

export const IMPORT_COLUMN_NAMES = {
  ticker: 'EW_Ticker',
  companyName: 'EW_CompanyName',
  groupName: 'EW_GroupName',
  notes: 'EW_Notes',
  enabled: 'EW_Enabled',
  groupPausedAt: 'EW_GroupPausedAt',
} as const

function rowsFromSheetJson(json: unknown[][]): ImportRow[] {
  if (json.length === 0) {
    throw new ImportFileError(
      'columns_not_found',
      'ファイルが空です。1行目に列見出しを入れてください。',
    )
  }
  const [headerRow, ...dataRows] = json
  const columns = headerRow.map((h) => HEADER_ALIASES[normalizeHeader(String(h ?? ''))])

  // ticker・companyNameのどちらの列も見つからない場合は、列見出し自体が想定と異なると判断する。
  const hasTicker = columns.includes('ticker')
  const hasCompanyName = columns.includes('companyName')
  if (!hasTicker && !hasCompanyName) {
    throw new ImportFileError(
      'columns_not_found',
      `列見出しが見つかりませんでした。1行目に ${IMPORT_COLUMN_NAMES.ticker} ・ ${IMPORT_COLUMN_NAMES.companyName} などの列名があるか確認してください。`,
    )
  }

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
  const lowerName = file.name.toLowerCase()
  const hasSupportedExtension = SUPPORTED_EXTENSIONS.some((ext) => lowerName.endsWith(ext))
  if (!hasSupportedExtension) {
    throw new ImportFileError(
      'unsupported_extension',
      `対応していないファイル形式です(${file.name})。CSV(.csv)またはExcel(.xlsx/.xls)ファイルを選択してください。`,
    )
  }

  let workbook: XLSX.WorkBook
  try {
    const buffer = await file.arrayBuffer()
    workbook = XLSX.read(buffer, { type: 'array' })
  } catch {
    throw new ImportFileError(
      'unreadable_file',
      'ファイルの内容を読み取れませんでした。ファイルが破損していないか確認してください。',
    )
  }

  const firstSheetName = workbook.SheetNames[0]
  const sheet = workbook.Sheets[firstSheetName]
  const json = XLSX.utils.sheet_to_json<unknown[]>(sheet, { header: 1, raw: false })
  return rowsFromSheetJson(json as unknown[][])
}
