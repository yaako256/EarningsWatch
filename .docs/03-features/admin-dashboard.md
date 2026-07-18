# EarningsWatch 本設計書 03-features. 管理者機能・ダッシュボード・ロギング

> `仮設計書.md` 10章(ロギング設計)・15章(フロントエンド設計のうちダッシュボード関連)を統合したもの。型定義は`02-types/api.md` 4・10章(管理者API・ダッシュボードAPI)、DDLは`01-db-schema.md` 1・8・9章を参照し、本ファイルでは業務ロジック・アーキテクチャを扱う。

## 目次
1. [ロギング設計(logging crate)](#1-ロギング設計logging-crate)
2. [ユーザダッシュボード表示内容](#2-ユーザダッシュボード表示内容)
3. [決算情報系画面(参考)](#3-決算情報系画面参考)
4. [管理者専用ダッシュボード](#4-管理者専用ダッシュボード)
5. [モバイル対応方針(参考、フロントエンド実装時の申し送り)](#5-モバイル対応方針参考フロントエンド実装時の申し送り)

---

## 1. ロギング設計(logging crate)

`tracing`クレートを用い、ログの発生箇所(呼び出し側)と処理方法(保存・通知)をLayerとして分離する。呼び出し側は`info!`/`warn!`/`error!`などのマクロでイベントを発行するだけでよい。

### 1.1 Layer構成
- `SqlLayer`: ログをSQLに保存(全体ロガー相当)
- `MemoryLayer`: warn/error以上をメモリに貯め、管理者が設定した通知先(`system_notify_config`、`01-db-schema.md` 9章)へ送信するトリガーを持つ(定期実行時ロガー相当)

登録するLayerはエントリポイントによって異なる。
- `server`起動時: `SqlLayer`のみ登録
- `cli monitor` / `cli notify`起動時: `SqlLayer` + `MemoryLayer`を登録

### 1.2 SQL保存(`SqlLayer`)の書き込み方式

`on_event`は同期関数のため直接`.await`できない。ログ発生頻度が高くなりうるため、都度同期的にDB書き込みへ行くのはレイテンシ・負荷両面で不利。よって非同期バッチ書き込み方式とする。

- `on_event`はログエントリをチャネルに送るのみ(非ブロッキング)
- 別タスクがチャネルを受信し、メモリバッファに蓄積した上でPostgreSQLへバルクINSERT

flush条件:
- n件溜まったら
- (`server`のみ)フロントエンドからログ表示のリクエストが来たら
- (`cli monitor` / `cli notify`のみ)単発実行が終わったら(最終flush)

`server`と`cli`は別プロセスのため、バッファ・flushタスクもプロセスごとに独立して存在する。

### 1.3 ログエントリに含める情報と取得方法

| 情報 | 取得方法 |
|---|---|
| ログレベル | `event.metadata().level()`から自動取得 |
| 発生箇所(ファイル/行/モジュール) | `event.metadata()`から自動取得 |
| 発生時刻 | 自動付与されないためLayer側で`Utc::now()`等を取得しセット |
| メッセージ・任意のフィールド内容 | `Visit`トレイトを実装し`event.record()`で走査して組み立て |
| どのプロセス由来か(server/monitor/notify) | Layer初期化時にプロセス種別を固定値として持たせ付与 |

「どのプロセス由来か」はDBのログテーブル(`logs.process`、`01-db-schema.md` 1章)にカラムとして明記する。

### 1.4 通知(`MemoryLayer`)の送信先

定期実行時(`cli monitor` / `cli notify`)にwarn/error以上が発生した場合、実行終了時にメモリバッファの内容を、管理者が設定した通知先(`system_notify_config`)へ送信する。

`server`側で見逃せない重大エラーが発生した場合も同じ通知先へ送信する経路を用意する。`server`は常駐プロセスのため「実行終了時にflush」という`cli`側のトリガーは適用できず、`MemoryLayer`は責務を分けて設計する。

- **バッファリング責務**: warn/error以上をメモリに貯める(`server`/`cli`共通)
- **flushトリガー責務**: いつ通知先へ送るかはプロセスの性質によって異なるため外部から注入できるようにする
  - `cli`(monitor/notify): プロセス終了時にflush
  - `server`: 1分程度(config管理)の短時間窓でまとめてflush。同一時間窓内に複数件のwarn/errorが発生した場合、まとめて1回の通知として送信する

monitor実行中の健全性チェック(マーカー行が残っている場合の警告)も、この`MemoryLayer`経由の通知経路を利用する(`01-db-schema.md` 6章参照)。

## 2. ユーザダッシュボード表示内容

想定表示項目(`GET /api/dashboard`、`02-types/api.md` 10章`DashboardResponse`)は以下の通り。

| 表示項目 | 内容 | 対応するレスポンスフィールド |
| --- | --- | --- |
| グループ数 | 自分が持つ`notify_groups`の件数 | `group_count` |
| フィルタ数 | 総フィルタ数 | `filter_count` |
| 送信媒体ごとの数 | discord/slackそれぞれのグループ数 | `medium_breakdown` |
| 一時停止中のグループ数 | `paused_at IS NOT NULL`の件数 | `paused_group_count` |
| webhook未設定のグループ数 | `notify_discord_configs.webhook_url IS NULL`等の件数 | `webhook_missing_count` |
| 直近送信 | `notify_history`から最新1件の詳細(グループ名・送信時刻等を含む) | `recent_sent` |
| 直近の送信失敗 | `notify_history`の`status = failed`のうち最新1件の詳細 | `recent_failed` |

> **本設計書で見つかった相違点**: `仮設計書.md` 15.1節の原案では、フィルタ数は「総フィルタ数」「ユニーク銘柄数(証券コード基準)」「ユニーク銘柄数(銘柄名基準)」の3行、直近送信・直近の送信失敗はそれぞれ「直近n件(n=10/n=5)」の一覧、加えて「直近n件の送信(グループごとの絞り込み)」を含む設計だった。一方`02-types/api.md`(型定義書由来)では、フィルタ数は単一の`filter_count`、直近送信・直近の送信失敗は各1件の詳細(`NotifyHistoryResponse`)のみとなっており、簡略化されている。この簡略化については追加点ファイル側に明示的な決定記録が見当たらなかった。「グループごとの絞り込み」は`GET /api/notify-history?group_id=...`(既存API)で代替可能なため実質的な機能欠落ではないが、**ユニーク銘柄数の内訳と、直近送信の複数件表示(n件一覧)は型定義書側で削られており、意図した簡略化か記載漏れかが不明**。本設計書末尾の「矛盾点・要確認事項」にまとめて報告する。

システム全体の実績(累計スクレイピング件数、送信成功率、最終監視実行時刻など)はユーザ単位の情報ではないため、ここには含めず管理者専用ダッシュボードとして別画面に切り出す(4章)。

## 3. 決算情報系画面(参考)

決算情報は株の情報であり全ユーザ共通で見られるものなので、ダッシュボードとは別画面として独立させる。

- 決算情報ログ(Excel出力可、ticker/company_name/evaluation/日付でフィルタ。`GET /api/earnings`・`GET /api/earnings/export`、`02-types/api.md` 5章)
- 決算集中度などのグラフ(`earnings`テーブルの`published_at`を日別集計して表示。`GET /api/earnings/summary`、同5章)

## 4. 管理者専用ダッシュボード

システム全体の実績・稼働状況を見る画面。ユーザダッシュボードとは完全に切り離す(`GET /api/admin/dashboard`、`02-types/api.md` 4章`AdminDashboardResponse`)。

| 表示項目 | 内容 |
| --- | --- |
| 累計スクレイピング件数 | `earnings`の`COUNT(*)`(または`system_runs.new_earnings_count`の`SUM()`) |
| 送信成功率 | `system_runs`(`run_type='notify'`)の直近n件から`SUM(success_send_count) / SUM(total_send_count)` |
| 最終監視実行時刻 | `system_runs`(`run_type='monitor'`)の`run_at`を最新1件取得 |
| 実行時間の推移 | `run_type`ごとに`duration_ms`を時系列で表示 |

管理者ログ一覧(`GET /api/admin/logs`)・仮ユーザ作成/無効化(`POST /api/admin/users`等)・定期実行ロガーの通知先設定(`GET/PUT /api/admin/notify-config`)も同じ管理者専用画面群に属する(`02-types/api.md` 4章参照)。

## 5. モバイル対応方針(参考、フロントエンド実装時の申し送り)

スマホ用UIは用意しない。スマホ(モバイル幅)でアクセスした場合は「スマホに対応していません。PCで開いてください。」という案内画面を表示する。それでも開きますか?という表示を出して、開けるようにもできるようにする。将来拡張として、スマホに対応したレスポンシブCSSを用意することを検討する(`05-future-work.md`)。

> 本項目はフロントエンド実装(`00-overview.md` 2章によりスコープ外)に関する内容だが、原本(`仮設計書.md` 15.4節)に記載されていた仕様であるため、将来のフロントエンド設計時の申し送り事項として本ファイルに記録しておく。
