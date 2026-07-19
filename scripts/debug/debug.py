#!/usr/bin/env python3
"""
scripts/debug/debug.py

固定値を返すダミースクレイパー。
Phase 13で実サイト(kabuyoho)スクレイピングに置き換えるまでの間、
monitor -> notify のパイプライン開発・動作確認に使う(design/00-overview.md 7.2章)。

Rust(scraperクレート)からは `python3 scripts/debug/debug.py list --page N` のように
呼び出され、stdout経由でJSONを1行出力する(design/03-features/scraping.md 12章)。
"""

import argparse
import json
import sys


def cmd_list(page: int) -> dict:
    """
    一覧ページ相当のダミーデータを返す。
    フィールド構成はdesign/03-features/scraping.md 11章の EarningItems 相当
    (fingerprint_item_1〜3 + url)。実際の判定ロジックはRust側が持つため、
    ここでは固定値を返すのみ。
    """
    # Phase 0時点では最小限の1件のみ返す雛形。
    # Phase 11でmonitor/notifyパイプラインを組む際、新規/既知判定を
    # 確認しやすい複数件・複数パターンのダミーデータに拡張する。
    return {
        "items": [
            {
                "fingerprint_item_1": "【ダミー】株式会社サンプル 決算速報",
                "fingerprint_item_2": "本日、株式会社サンプルは決算を発表しました。",
                "fingerprint_item_3": "UNRATED",
                "url": "https://example.com/debug/detail/1",
            }
        ]
    }


def cmd_detail(url: str) -> dict:
    """個別ページ相当のダミーデータを返す(Earnings構造体相当のフィールド構成)。"""
    return {
        "ticker": "0000",
        "company_name": "株式会社サンプル",
        "published_at": "2026-01-01T00:00:00Z",
        "title": "【ダミー】株式会社サンプル 決算速報",
        "url": url,
        "summary": "本日、株式会社サンプルは決算を発表しました。",
        "evaluation": "UNRATED",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list")
    list_parser.add_argument("--page", type=int, required=True)

    detail_parser = subparsers.add_parser("detail")
    detail_parser.add_argument("--url", type=str, required=True)

    args = parser.parse_args()

    if args.command == "list":
        output = cmd_list(args.page)
    elif args.command == "detail":
        output = cmd_detail(args.url)
    else:
        print(f"unknown command: {args.command}", file=sys.stderr)
        sys.exit(1)

    print(json.dumps(output, ensure_ascii=False))


if __name__ == "__main__":
    main()