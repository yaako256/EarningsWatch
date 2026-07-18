# EarningsWatch 本設計書 03-features. 通知送信

> `仮設計書.md` 8章を土台に、`仮設計書からの追加点と変更点.md`のTODO#1補足(Slack先送り方針)・TODO#3解決(mention_targets記法変換ロジック)・関連補足・追加仕様(テスト送信プレビュー拡張)を統合したもの。型定義そのものは`02-types/notifier.md`・`02-types/subscription.md`を参照し、本ファイルでは業務ロジック・仕様を扱う。

## 目次
1. [送信先(通知先)](#1-送信先通知先)
2. [フィルタ・グループの枠組み](#2-フィルタグループの枠組み)
3. [フィルタ設定方法](#3-フィルタ設定方法)
4. [送信先の指定](#4-送信先の指定)
5. [用語の整理(混乱防止)](#5-用語の整理混乱防止)
6. [`notify_filters`の重複行の扱い・一覧表示方針](#6-notify_filtersの重複行の扱い一覧表示方針)
7. [通知媒体固有設定](#7-通知媒体固有設定)
8. [mention_targetsの記法変換ロジック(TODO#3解決)](#8-mention_targetsの記法変換ロジックtodo3解決)
9. [notify実行時のグループ別フィルタリングの仕組み](#9-notify実行時のグループ別フィルタリングの仕組み)
10. [個別送信失敗時のリトライ](#10-個別送信失敗時のリトライ)
11. [送信先webhookのテスト送信・プレビュー機能拡張](#11-送信先webhookのテスト送信プレビュー機能拡張)

---

## 1. 送信先(通知先)

現段階では以下を想定する。
- Discord
- Slack(詳細仕様はMVP内拡張フェーズで決定、7章参照)

## 2. フィルタ・グループの枠組み

- ユーザごとに設定
- グループを作れる
- グループごとに送信先も異なる可能性あり
- 銘柄名の揺れも考慮し、証券コードと銘柄名の両方でフィルタ。片方一致で送信内容に含める

## 3. フィルタ設定方法

- フロントエンドで設定
- 視覚的な設定方法とCSV/JSONなどコード的な設定方法を想定
- CSV/ExcelのDrag & Dropによる一括設定も想定(ユーザ全体向け・グループ単体向けの2種類、詳細は`03-features/import-export.md`)

## 4. 送信先の指定

- グループ毎の送信先指定はフロントエンドの視覚的操作で設定可能
- 送信先の一括指定も可能にする(`PUT /api/groups/bulk-destination`、`02-types/api.md` 7章)
- Discordの場合は専用設定項目を用意(Embed使用有無、色など)

## 5. 用語の整理(混乱防止)

「一括設定」という言葉がCSV由来とフロントエンド由来の2種類で使われ紛らわしいため、以下のように呼称を分ける。

- **フィルタ一括インポート**:CSV/ExcelのDrag & Dropによる、フィルタ内容(証券コード・銘柄名など)の一括登録・更新(`03-features/import-export.md`)
- **送信先一括設定**:フロントエンドの視覚的操作による、複数グループへの送信先設定の一括反映(本ファイル4章)

## 6. `notify_filters`の重複行の扱い・一覧表示方針

同一グループ内で同じ`ticker`+`company_name`が重複登録されても、UNIQUE制約は設けずエラーは出すが登録自体は許可する(`01-db-schema.md` 4章)。

- 動作に支障がないため
- ユーザが同じ銘柄に複数の備考(`notes`)を残したい場合があるため
- 後から変更が容易なため

フィルタ一括インポート時も同様の方針とし、重複行があってもインポート自体は継続する。インポート結果画面またはログで「n件重複の可能性あり」等の警告を表示する(`03-features/import-export.md`)。

**フィルタ一覧表示の方針**:重複行の見分けがつくよう、フィルタ一覧画面では`ticker`/`company_name`/`notes`を含む全カラムを常に表示する(`notes`を隠さない)。これにより、同一銘柄の重複行がある場合も`notes`の内容で見分けがつくようにする。

## 7. 通知媒体固有設定

サブタイプは、sqlxの型安全を利かせるため媒体ごとに専用テーブルを作る方針。処理上は証券コードと銘柄名の片方が埋まっていればいいが、見た目上どちらも必須とする。フロントエンドで送信媒体などの設定を切り替えても設定を残すため、グループ作成時に全媒体テーブルへ必ず1行作成し以後残し続ける(`01-db-schema.md` 4章)。グループ一括設定の項目はDBに持たせず、フロントエンド側の機能として実現する。

- **Discord固有設定**:`webhook_url`、`embed_color`(16進カラーコード文字列)、`mention_enabled`、`mention_targets`(配列)。型定義は`02-types/notifier.md`の`DiscordConfig`
- **Slack固有設定**:仮カラムとして`webhook_url`・`mention_enabled`・`mention_targets`を暫定的に持つ(`02-types/notifier.md`の`SlackConfig`)。Slack Incoming Webhook / Block Kitの仕様確認後に確定する

### Slack詳細仕様の着手方針(TODO#1解決)

Slack通知の詳細仕様(`notify_slack_configs`のカラム構成、webhookの記法、`mention_targets`の記法等)は、Discordの実装が固まった後にMVP内の拡張として着手する。

- **Discordを基準に先行実装する**。8章・TODO#3で決定した設計(`notify_discord_configs`のカラム構成、`mention_targets`の`type:value`形式、`allowed_mentions`を用いた変換ロジック)をまず完成させる
- **Slackの詳細仕様はMVP内の拡張フェーズで決定する**。将来拡張(`05-future-work.md`)には回さず、あくまでMVP内でDiscordの後に続けて実装する
- Slack実装着手時に、以下を本項目の解消として調べ直す
  - `notify_slack_configs`のカラム構成(Discordの`webhook_url`/`embed_color`/`mention_enabled`/`mention_targets`に相当する項目をSlackでどう持つか)
  - Slack Incoming Webhookのペイロード記法、Block Kitを使うか否か
  - `mention_targets`のSlack側記法(`@here`/`@channel`/ユーザID等)とDiscord方式(`type:value`文字列、例:`user:123`)との対応関係
- 着手タイミングは「Discordでの通知パイプライン(monitor→notify)が一通り動作確認できた後」を目安とする

## 8. mention_targetsの記法変換ロジック(TODO#3解決)

Discord向けのmention_targets記法変換を先に決定する。Slackは今回対象外とし、MVP内でDiscord実装後に拡張する際に別途決定する(7章のTODO#1として引き続き扱う)。

### mention_targetsの保存形式

DBは`TEXT[]`のまま。要素は以下のtypeを持つ文字列とする。

| type | 形式 | 入力方式 |
|---|---|---|
| user | `user:<discord_user_id>` | フロントで自由入力(IDをそのまま貼り付け) |
| role | `role:<discord_role_id>` | フロントで自由入力(IDをそのまま貼り付け) |
| 特殊系(メンション) | `everyone` / `here` | フロントで選択式(enum的なUIで誤入力を防止) |
| 特殊系(タイムスタンプ) | `time:<style>` | フロントで選択式。styleは以下7種から選択 |

コロンの有無・プレフィックスで機械的にtype判別が可能(`user:`/`role:`/`time:`プレフィックスがあればID・パラメータ系、なければ`everyone`/`here`の固定値)。

**入力方式の補足**:Discordのユーザ/ロールIDはコピー&ペーストで入力できるため、過去入力のサジェスト機能等は設けず、自由入力欄のままとする。

### timeスタイル一覧(7種全対応)

| style | 表示例 | 用途 |
|---|---|---|
| `t` | 4:20 PM | 短い時刻 |
| `T` | 4:20:30 PM | 長い時刻 |
| `d` | 3/6/2026 | 短い日付 |
| `D` | March 6, 2026 | 長い日付 |
| `f` | March 6, 2026 4:20 PM | 短い日付時刻(Discordのデフォルト) |
| `F` | Friday, March 6, 2026 4:20 PM | 長い日付時刻 |
| `R` | in 2 hours | 相対時間(Discord側でリアルタイム更新) |

`TimeStyle`の型定義は`02-types/notifier.md`参照。

### Discord変換ロジック(notify処理側で実装)

| mention_targets要素 | Discordメッセージ本文への変換 | allowed_mentionsへの追加 |
|---|---|---|
| `user:123` | `<@123>` | `users: ["123", ...]` に追加 |
| `role:456` | `<@&456>` | `roles: ["456", ...]` に追加 |
| `everyone` | `@everyone` | `parse: ["everyone"]` に追加 |
| `here` | `@here` | `parse: ["everyone"]` に追加(`@here`も`everyone`パースフラグで許可される) |
| `time:<style>` | `<t:<unix_timestamp>:<style>>`(送信時刻をUnix秒に変換して埋め込み) | 対象外(メンションではないため`allowed_mentions`不要) |

**注意**:Discordはデフォルトでcontent内の全メンションを発火させてしまうため、`allowed_mentions`で明示的に許可リストを組む必要がある(未指定分は`parse: []`でブロックする安全設計とする)。`time:<style>`はメンションではなく表示形式の埋め込みのため、`mention_enabled`のON/OFFやallowed_mentionsの対象外として扱う。

不正な`mention_targets`要素(未知のプレフィックス等)は警告ログを残しつつ当該要素のみスキップし、送信自体は続行する(`02-types/notifier.md` 1章)。

### Slackについて

今回は対象外。Discord実装完了後、MVP内で拡張実装する際にSlackの記法(Block Kit等)を調査し決定する(7章のTODO#1として引き続き扱う)。

## 9. notify実行時のグループ別フィルタリングの仕組み

`notify_queue`は決算単位(グループ横断で共通)で1行持つ設計(`01-db-schema.md` 6章)だが、実際の配信可否はグループごとのフィルタ条件で決まる。この判定の流れを明記する。

- `monitor`実行時、`notify_queue`には**新規に検出された決算すべて**を無条件で登録する(グループのフィルタ条件による絞り込みはこの時点では行わない)
- `notify`実行時、登録されている**全グループを順々に処理**し、各グループに紐づく`notify_filters`(ticker/company_name)でフィルタリングしながら送信する
- つまり「どのグループへ配信するか」の判定は`notify`実行時、グループを1つずつ舐める形で行われる(`notify_queue`側にグループ紐付けの列は持たない)

## 10. 個別送信失敗時のリトライ

送信が`failed`になった場合、数分待って再送信を試みるリトライを複数回行う。

- このリトライは**当該notify実行内で完結**し、次回notify実行には持ち越さない(次回notify実行時のmonitor健全性チェック、`01-db-schema.md` 6章とは独立したロジック)
- 最終的にリトライしても失敗した場合は、`notify_history`に`failed`として記録される(`ApiErrorCode::NotifySendFailed`/`NotifyRejected`、`02-types/api.md` 2章をそのまま利用)

## 11. 送信先webhookのテスト送信・プレビュー機能拡張

既存のテスト送信(`POST /api/groups/{id}/config/test-send`)を拡張し、送信内容をユーザが自由入力してプレビューできるようにする。新規API・新規テーブルは追加せず、既存エンドポイントのリクエストボディを拡張するのみとする(型定義は`02-types/api.md` 7章`TestSendRequest`)。

- 実際にDiscordへ送信する方式とする(フロントエンド内だけの再現ではなく、実送信によってロールメンション等の見た目も確認できるようにする)
- リクエストボディに以下を**すべて任意項目**として追加。フロントエンドは各入力欄にデフォルト値をプレースホルダー(薄灰色)として表示し、未入力ならデフォルト値を、入力があればその値を送信する

| フィールド | 空欄時のデフォルト |
|---|---|
| `ticker` | 固定のダミー値 |
| `company_name` | 固定のダミー値 |
| `title` | 固定のダミー値 |
| `evaluation` | `Unrated` |
| `embed_color` | グループ設定済みの`embed_color` |
| `webhook_url` | グループに保存済みのwebhook_url(一時上書き可能。誤って本番サーバへ送りたくない場合に個人用テストサーバのURLへ差し替え可能) |
| `mention_targets` | グループに保存済みのmention_targets(一時上書き可能。ロールメンションの見た目確認用途) |

- `webhook_url`・`mention_targets`を上書きした場合も**DBには保存しない**(その場限りの一時送信であり、グループの保存済み設定は変更しない)
- バックエンドはリクエストされた値をそのまま`MonitoredEarningsReport`相当の形に組み立てて`notifier`crateへ渡すだけで済むため、実装コストは低い
- HTTPレベルでの送信成否のみを判定する(`TestSendResponse`)。「意図した送信先に届いたか」の検証はユーザの責任範囲とする(`02-types/api.md` 7章参照)
