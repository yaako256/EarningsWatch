#!/usr/bin/env python3
"""
scripts/kabuyoho/kabuyoho.py

kabuyoho.jpの決算速報一覧・個別ページをPlaywrightでスクレイピングする。
Rust(scraperクレート、KabuyohoScraper)から
`python3 scripts/kabuyoho/kabuyoho.py list --page N` /
`python3 scripts/kabuyoho/kabuyoho.py detail --url URL`
の形式で呼び出され、stdout経由でJSONを1行出力する。

設計方針:
- 1ファイルで完結させる。定義・設定系はファイル上部に集約する。
- Rust側がページ間隔の待機(list/detail interval)を行うため、本スクリプト内では待機しない。
  ただし人間らしさを持たせるため、実行のたびに0〜3秒のジッターのみ行う。
- サイト構造が変わって要素が見つからない場合は、空データ扱いにしてstderrへ警告を出す
  (stdoutのJSONを壊さないため、警告はstderrにのみ書く)。
"""

import argparse
import json
import random
import sys
import time
from datetime import datetime, timedelta, timezone
from urllib.parse import urljoin

from playwright.sync_api import sync_playwright

# ===== 定義・設定(変更が必要な場合はここを直す) =====

BASE_URL = "https://kabuyoho.jp"
LIST_PATH = "/consNewsList"
LIST_QUERY_CAT = "1"  # 業績速報(スクレイピング決定事項.md「cat」参照)
LIST_QUERY_LST = ""   # 空 = 直近1週間(スクレイピング決定事項.md「lst」参照)

# 一覧ページのセレクタ
LIST_CONTAINER_SELECTOR = "ul.news_list"
LIST_ITEM_SELECTOR = "li"
LIST_ITEM_LINK_SELECTOR = "a"
LIST_ITEM_SUMMARY_SELECTOR = "p"

# 個別ページのセレクタ
DETAIL_ARTICLE_SELECTOR = "section.news_article"
DETAIL_SUMMARY_BLOCK_SELECTOR = "div.article_smry"
DETAIL_TIME_SELECTOR = "span.time"
DETAIL_PDF_LINK_SELECTOR = "a.file.file_pdf"
DETAIL_BRAND_BLOCK_SELECTOR = "div.brand_name"
DETAIL_BRAND_NAME_SELECTOR = "a.name"
DETAIL_BRAND_CODE_SELECTOR = "span.nmbr"
DETAIL_ARTICLE_BODY_SELECTOR = "article"
DETAIL_TITLE_SELECTOR = "h2"
DETAIL_SUMMARY_TEXT_SELECTOR = "p"

# 決算評価のCSSクラス→EarningsEvaluation対応
WTHR_CLASS_TO_EVALUATION = {
    "wthr_fine": "POSITIVE",
    "wthr_clud": "NEUTRAL",
    "wthr_rain": "NEGATIVE",
    "wthr_empt": "UNRATED",
}
DEFAULT_EVALUATION = "UNRATED"

JITTER_MIN_SECONDS = 0.0
JITTER_MAX_SECONDS = 3.0

JST = timezone(timedelta(hours=9))


# ===== ユーティリティ =====

def jitter():
    """ジッター処理(0〜3秒ランダム)"""
    time.sleep(random.uniform(JITTER_MIN_SECONDS, JITTER_MAX_SECONDS))


def warn(message: str):
    """stdoutのJSONを壊さないよう、警告はstderrにのみ出す"""
    print(f"[kabuyoho] {message}", file=sys.stderr)


def wthr_class_to_evaluation(class_attr: str) -> str:
    """class="wthr wthr_fine"のような文字列から評価を判定する"""
    for cls in class_attr.split():
        if cls in WTHR_CLASS_TO_EVALUATION:
            return WTHR_CLASS_TO_EVALUATION[cls]
    return DEFAULT_EVALUATION


def parse_jst_datetime_to_utc(text: str) -> str:
    """"2026/07/23 16:00"のようなJST表記をUTCのISO8601文字列へ変換する。"""
    dt_jst = datetime.strptime(text.strip(), "%Y/%m/%d %H:%M").replace(tzinfo=JST)
    dt_utc = dt_jst.astimezone(timezone.utc)
    return dt_utc.strftime("%Y-%m-%dT%H:%M:%SZ")


# ===== 一覧ページ =====

def build_list_url(page: int) -> str:
    return f"{BASE_URL}{LIST_PATH}?cat={LIST_QUERY_CAT}&lst={LIST_QUERY_LST}&page={page}"


def cmd_list(page: int) -> dict:
    jitter()
    url = build_list_url(page)
    items = []

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        pw_page = browser.new_page()

        try:
            pw_page.goto(url, timeout=30000)
            pw_page.wait_for_selector(LIST_CONTAINER_SELECTOR, timeout=10000)
        except Exception:
            # 一覧要素が見つからない(ページ切れ、サイト構造変化のいずれか) → 空データ扱い
            warn(f"一覧コンテナ({LIST_CONTAINER_SELECTOR})が見つかりませんでした: page={page}")
            browser.close()
            return {"items": []}

        list_items = pw_page.query_selector_all(f"{LIST_CONTAINER_SELECTOR} {LIST_ITEM_SELECTOR}")

        for li in list_items:
            try:
                link = li.query_selector(LIST_ITEM_LINK_SELECTOR)
                summary_el = li.query_selector(LIST_ITEM_SUMMARY_SELECTOR)

                if link is None or summary_el is None:
                    warn("一覧の1件でリンクまたは要約が取得できませんでした。スキップします。")
                    continue

                href = link.get_attribute("href") or ""
                class_attr = link.get_attribute("class") or ""
                title = link.inner_text().strip()
                summary = summary_el.inner_text().strip()

                items.append({
                    "fingerprint_title": title,
                    "fingerprint_summary": summary,
                    "fingerprint_evaluation": wthr_class_to_evaluation(class_attr),
                    "url": urljoin(BASE_URL, href),
                })
            except Exception as e:
                warn(f"一覧の1件の解析に失敗しました。スキップします: {e}")
                continue

        browser.close()

    return {"items": items}


# ===== 個別ページ =====

def cmd_detail(url: str) -> dict:
    jitter()

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        pw_page = browser.new_page()

        try:
            pw_page.goto(url, timeout=30000)
            pw_page.wait_for_selector(DETAIL_ARTICLE_SELECTOR, timeout=10000)
        except Exception:
            warn(f"個別ページのコンテナ({DETAIL_ARTICLE_SELECTOR})が見つかりませんでした: url={url}")
            browser.close()
            return {
                "ticker": "",
                "company_name": "",
                "published_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
                "title": "",
                "url": "",
                "summary": "",
            }

        article = pw_page.query_selector(DETAIL_ARTICLE_SELECTOR)

        # 公開時刻
        smry_block = article.query_selector(DETAIL_SUMMARY_BLOCK_SELECTOR)
        time_el = smry_block.query_selector(DETAIL_TIME_SELECTOR) if smry_block else None
        published_at = (
            parse_jst_datetime_to_utc(time_el.inner_text())
            if time_el is not None
            else datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        )
        if time_el is None:
            warn("公開時刻(span.time)が取得できませんでした。現在時刻で代用します。")
        
        # 公式声明PDFへのリンク
        pdf_el = smry_block.query_selector(DETAIL_PDF_LINK_SELECTOR) if smry_block else None
        pdf_href = pdf_el.get_attribute("href") if pdf_el is not None else None
        pdf_url = urljoin(BASE_URL, pdf_href) if pdf_href else ""
        if pdf_el is None:
          warn("公式声明PDFへのリンク(a.file.file_pdf)が取得できませんでした。")

        # 銘柄情報
        brand_block = article.query_selector(DETAIL_BRAND_BLOCK_SELECTOR)
        name_el = brand_block.query_selector(DETAIL_BRAND_NAME_SELECTOR) if brand_block else None
        code_el = brand_block.query_selector(DETAIL_BRAND_CODE_SELECTOR) if brand_block else None
        company_name = name_el.inner_text().strip() if name_el is not None else ""
        ticker = code_el.inner_text().strip() if code_el is not None else ""
        if name_el is None or code_el is None:
            warn("銘柄情報(div.brand_name)が一部取得できませんでした。")

        # 本文
        body = article.query_selector(DETAIL_ARTICLE_BODY_SELECTOR)
        title_el = body.query_selector(DETAIL_TITLE_SELECTOR) if body else None
        summary_el = body.query_selector(DETAIL_SUMMARY_TEXT_SELECTOR) if body else None
        # inner_text()はHTMLタグを剥がしたテキストを返すため、<a>タグ混在の除去も自動的に満たされる
        title = title_el.inner_text().strip() if title_el is not None else ""
        summary = summary_el.inner_text().strip() if summary_el is not None else ""
        if title_el is None or summary_el is None:
            warn("本文(article内のh2/p)が一部取得できませんでした。")

        browser.close()

    return {
        "ticker": ticker,
        "company_name": company_name,
        "published_at": published_at,
        "title": title,
        "url": pdf_url,
        "summary": summary,
    }


# ===== エントリポイント =====

def main():
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