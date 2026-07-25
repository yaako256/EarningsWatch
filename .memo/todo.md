## todo

バックエンドがほとんど完成した。
次はphase13の、実際のスクレイピング処理実行。
しかし、Discordへの出力がキモかったりで、色々修正したい部分がある。
一旦スクレイピングの項目をまとめ、claudeに投げる。
その後、修正項目を修正し、それを実装する。

そしたらフロントエンド作成に進む

実際のスクレイピング処理とかも含め、バックエンドMVPが完成した。
Claudeのトークンと相談しながらAPI設計書などを作り、Codexで一気にフロントエンドを仕上げる。
僕はバックエンドの修正項目を修正していく。




# requirements.txt
requirements.txtを作らなきゃいけん
後で作る。
現在はrequirements.txtをYutaiWatchからコピってくることで解決しており、devステージでもrequirements.txtを使っている。

# Makefile群 → 多分大体対処済み
SQLテーブルや、cliコマンド、migrationパス(そもそもいらないかも)などがYaakoDriveのままとなっている。
実際に使うときになったら編集する。

# composeのport → EarningsWatchは修正した。バックエンド：本番は8000番台、開発は9000番台。フロントエンド：本番は3000番台、開発は9000番台
今後おかしくなる可能性がある。
YaakoDrive含め、ポートの整理をした方がいいかもしれない。
開発用ポートを何番にするとか、10単位でプロジェクトを分けるとか。

# ログ何件溜まったら
configへ切り出す

# リポジトリクレートとinfraクレート → コメントとかはつけてないけど確認はしたため、解決済み
詳細なコメントとかつけ取らん。
ブラックボックスじゃないけどグレーボックス


# infraクレートのSlack設定 → やった。解決済み
Discordはクエリの共通化をしているが、Slackはしていない。
Slack実装時にそれを変更しなければならないかもしれない。


## apiのjson型について → 漏れはあるかもしれないがつけた
`#[serde(rename_all = "lowercase")]`を付けていかなければいけないかもしれない。
↓
`#[serde(rename_all = "camelCase")]`だった。
JsonAPI系は`camelCase`よく使われる
以下にする。
```
Rust内部: snake_case(ものにより)
DBカラム: snake_case(ものにより)
Query Parameter: snake_case(絶対)
JSON Body: camelCase(絶対)
```

# init/monitor実行時のログ
処理が長くて心配になるから個別ページはn/m番目って表示させたいかも。