# EarningsWatch 本設計書 04. 認証・権限・セキュリティ・設定管理

> `仮設計書.md` 11〜12章を土台に、`仮設計書からの追加点と変更点.md`のTODO#6解決(本番Cookie SecureフラグとTailscale運用の整合性)・TODO#7解決(webhook_url暗号鍵のローテーション運用)を統合したもの。暗号化型の定義自体は`02-types/crypto.md`を参照。

## 目次
1. [JWT/Cookieのフロー](#1-jwtcookieのフロー)
2. [Cookie設定(TODO#6解決反映済み)](#2-cookie設定todo6解決反映済み)
3. [管理者ユーザとそれ以外の権限差](#3-管理者ユーザとそれ以外の権限差)
4. [仮ユーザ作成フロー](#4-仮ユーザ作成フロー)
5. [webhook_url等の機密情報の扱い](#5-webhook_url等の機密情報の扱い)
6. [webhook_url暗号鍵のローテーション運用(TODO#7解決)](#6-webhook_url暗号鍵のローテーション運用todo7解決)
7. [通信経路の保護](#7-通信経路の保護)
8. [バリデーション方針](#8-バリデーション方針)
9. [設定ファイル・環境変数の命名規則](#9-設定ファイル環境変数の命名規則)

---

## 1. JWT/Cookieのフロー

別プロジェクト(YaakoDrive)の設計をそのまま流用する。

- アクセストークン有効期限: 15分(config管理)
- リフレッシュトークン有効期限: 30日(config管理)
- リフレッシュタイミング: リクエストが401を受けたらリフレッシュを実行し、元のリクエストを再試行する(先読み更新はしない)
- 認証エラー時のフロー:`/api/auth/refresh`が失敗した場合(リフレッシュトークン期限切れ・revoke済みなど)は`401 unauthorized`を返す。フロントエンドはこれを受けたらログイン状態を破棄し、ログイン画面へ遷移する(フロントエンド実装自体は`00-overview.md` 2章によりスコープ外だが、バックエンド側が前提とする挙動として記載する)

## 2. Cookie設定(TODO#6解決反映済み)

Tailscale閉域内での運用はHTTP接続のため、Secure=trueにするとブラウザがCookieを送信せずログインが機能しない。既存運用中のYaakoDriveで同様の問題に直面した実績を踏まえ、以下の方針とする。

- 本番・開発ともに、Secureはconfigで環境ごとに切り替え可能な設定のままとする
- 現時点では**本番・開発の両方でSecure=false**とする(Tailscale自体がWireGuardで通信を暗号化しているため、アプリ側の追加保護は現状不要と判断。7章の通信経路保護の方針と整合)
- 将来HTTPS化(`05-future-work.md`)が完了した時点で、**本番のみSecure=trueへ切り替える**(開発はvite等の都合上false運用を継続する想定)

| 属性 | 本番 | 開発 | 理由 |
|------|------|------|------|
| HttpOnly | true | true | JSから読めなくする |
| Secure | **false**(HTTPS化後にtrueへ切替) | false | Tailscale閉域内はHTTP運用のため、trueにするとCookieが送信されずログイン不能になる。HTTPS化後は本番のみtrueに切り替える |
| SameSite | Strict | Strict | CSRF対策 |
| Path(Access Token) | `/api` | `/api` | API全体で利用 |
| Path(Refresh Token) | `/api/auth/refresh` | `/api/auth/refresh` | Refresh Token送信範囲を限定 |

## 3. 管理者ユーザとそれ以外の権限差

個人・家族・友人向けの小規模運用のため、シンプルな2階層(admin/user)とする。グループ数・フィルタ数などの利用制限は設けない(全ユーザ共通で無制限)。

### 管理者(admin)ができること

| 機能 | 内容 | 対応API |
| --- | --- | --- |
| ログ確認 | 全ユーザ分のログを閲覧(日時範囲・レベル・プロセスでのフィルタ、ページング) | `GET /api/admin/logs` |
| ユーザ確認 | ユーザ一覧・各ユーザの利用状況(グループ数/フィルタ数/媒体種別)を閲覧 | `GET /api/admin/users`, `GET /api/admin/users/{id}/summary` |
| 仮ユーザ作成 | username指定+ランダム仮パスワード自動生成でユーザを作成 | `POST /api/admin/users` |
| ユーザ無効化(BAN) | 問題があった場合にユーザを無効化(`disabled_at`セット) | `POST /api/admin/users/{id}/disable` |
| 定期実行ロガーの通知先設定 | warn/error通知先(Discord等)の設定。管理者全体で共有する1設定(`01-db-schema.md` 9章) | `GET/PUT /api/admin/notify-config` |
| お知らせ板・固定ページの作成/編集/削除 | `03-features/notice-board.md`参照 | `POST/PUT/DELETE /api/pages/{id}` |

### 一般ユーザ(user)ができること

| 機能 | 内容 | 対応API |
| --- | --- | --- |
| 自分のユーザ名・パスワード変更 | 仮アカウントから本アカウントへの移行(任意、強制しない) | `PUT /api/users/me/username`, `PUT /api/users/me/password` |
| グループ・フィルタ・送信先の管理 | 制限なし | 各種グループ/フィルタAPI |

### 管理者が閲覧できない情報

フィルタの中身(ticker/company_name等)は個人の監視対象情報であるため、管理者であっても閲覧不可。集計値(グループ数・フィルタ数・媒体種別)のみ閲覧可能とする(`02-types/api.md` 4章`UserSummaryResponse`)。

## 4. 仮ユーザ作成フロー

1. 管理者がフロントエンドで仮ユーザ作成を実行(usernameは管理者が指定、パスワードはランダム自動生成)
2. 生成された仮パスワードは管理者画面に**一度だけ表示**(再表示不可。DBにはハッシュのみ保存。`02-types/api.md` 4章`CreateUserResponse`)
3. 管理者が口頭・チャット等でユーザ本人に仮パスワードを伝える
4. ユーザ名・パスワードの変更は**任意**(強制しない。変えなくても使えるが、変えたくなる程度にはランダムな仮パスワードとする)

## 5. webhook_url等の機密情報の扱い

webhook_urlは知られると第三者が任意に送信できてしまう認証情報(シークレット)であるため、以下の方針とする。

- DBには**アプリ層で暗号化**した状態で保存する(`crypto`クレートの`Encrypted<T>`、`02-types/crypto.md`)
- 復号は`notify`処理(実際に送信するタイミング)、および`GET /api/groups/{id}/config`のレスポンス生成時に行う
- APIレスポンスではマスクせず、復号した値をそのまま返す(ユーザ本人が設定内容を確認できる必要があるため。目視での盗み見は脅威モデルに含めない)

### 暗号化方式

- アルゴリズム: AES-256-GCM
- 鍵: 32byteの鍵をconfigで管理し、環境変数(`EARNINGSWATCH__SECURITY__WEBHOOK_ENC_KEY`、base64エンコード)で上書きする
- Nonce: 暗号化ごとに12byteのランダム値を生成し、暗号文の先頭に付与した上でまとめてbase64化してTEXTカラムに保存する(`nonce || ciphertext`形式)
- AAD(付加認証データ): `group_id`(グループ固有設定)または管理者共有設定である旨を示す固定値(`system_notify_config`、`SystemNotifyWebhookUrlTag`)を付与し、暗号文が別グループ・別用途のレコードへ転用されてもGCM認証エラーとして検知できるようにする
- 鍵のローテーション運用は現時点では設計しない(6章、将来拡張へ)

## 6. webhook_url暗号鍵のローテーション運用(TODO#7解決)

鍵ローテーションは本来「漏洩などのインシデント発生時に緊急対応として行うもの」であり、個人・家族・友人向けの小規模運用(`00-overview.md` 4章原則7)において予防的な定期ローテーションを常設する必要性は薄いと判断した。

### 決定内容

- **MVPでは鍵ローテーション機能を実装しない**。5章で定めた単一の固定鍵(config/環境変数`EARNINGSWATCH__SECURITY__WEBHOOK_ENC_KEY`管理)による運用を継続する
- 鍵ローテーションの仕組み(`cli rotate-webhook-key`等のコマンド、旧鍵→新鍵での全レコード再暗号化バッチ)は実装せず、**`05-future-work.md`「将来拡張」に追加**する
- 将来実装する際も、cronによる定期実行ではなく、漏洩などのインシデント発生時に手動でcliを実行する運用を基本とする方針をメモしておく

> 実装する場合は緊急時対応(手動実行)を基本とし、cronによる定期実行は想定しない。新しい鍵の生成方法・configへの反映方法・再暗号化バッチの冪等性については実装時に別途設計する。

## 7. 通信経路の保護

本番運用はTailscale経由でのアクセスに閉じるため、Tailscale自体が通信を暗号化している。そのため、パスワード・ユーザ名についてアプリ側で追加の暗号化通信対応は行わない。

- 検討はしたが、Tailscaleにより通信はすでに暗号化されているため、アプリ側での処理は不要と判断した
- 将来、一般公開する場合に備え、HTTPS化は`05-future-work.md`の将来拡張として記載する

## 8. バリデーション方針

| 項目 | ルール |
| --- | --- |
| `ticker` | 空文字不可のみ(パターンチェックはしない) |
| `company_name` | 空文字不可のみ |
| `notes` | 任意(空文字許容) |
| グループ`name` | 空文字不可、**1〜30文字**(仮決定)。個人・家族・友人向け利用で表示上見やすい長さとして設定。DiscordのEmbedタイトル等に載せても崩れない長さを想定 |
| `webhook_url` | 空文字は許容(未設定として扱う)。値がある場合はURL形式チェックのみ |
| `embed_color` | `NULL`ならデフォルト色として判定。フォーマットは16進カラーコード文字列(例: `0x87EB87`)に確定。フロントエンドではこの文字列を直接入力させず、0〜255のRGBスライダー等による視覚的な色選択UIを用意し、選択結果をこの文字列形式に変換して送信する。デフォルト色は水色(`0x87CEEB`)とし、configで設定する(`02-types/notifier.md`の`EmbedColor::DEFAULT`) |
| `mention_targets` | 空配列許容(メンションなしとして扱う)。`mention_enabled = true`かつ配列が空の場合は単にメンションなしとして送信する(フロントエンド側で、その状態で保存できなくするバリデーションを想定)。Discord/Slackそれぞれの記法への変換は`notify`処理側の実装詳細(`03-features/notification.md` 8章)とする |

## 9. 設定ファイル・環境変数の命名規則

環境ラベルでの区別方針とする。

```text
config/
├── .env.dev
├── .env.prod
├── config.dev.toml
└── config.prod.toml
```

環境変数は`EARNINGSWATCH__...`形式で階層的に上書きできるようにする。

```text
EARNINGSWATCH__DATABASE__URL=postgres://...
EARNINGSWATCH__JWT__ACCESS_TOKEN_TTL_MINUTES=15
EARNINGSWATCH__JWT__REFRESH_TOKEN_TTL_DAYS=30
EARNINGSWATCH__COOKIE__SECURE=false
EARNINGSWATCH__SECURITY__WEBHOOK_ENC_KEY=...
EARNINGSWATCH__LOGGING__SERVER_FLUSH_WINDOW_SECONDS=60
```

`Cargo.toml`はRust依存関係・ビルド設定用であり、アプリ設定には使わない。
