# 開発日記

# 設計書の作成
2026年7月11日から2026年7月18日までの1週間は設計書の作成をしていた。

# 2026年7月19日
- 開発指南作成中
- phase0のリポジトリ土台完成。いつでもバックエンド開発に取り組める状態に。
  * backend (Rust)
  * frontend (React)
  * Makefile群
  * Docker系統
  * configひな形
  * ダミースクレイパー`scripts/debug/debug.py`
- phase1のworkspace土台を完成。
- phase2のconfigクレートと、設定ファイルの整備が完了
- phase3のDB/migration関連が完了(cliはmigrationだけを実行するものになっている)
- phase4のドメイン型の作成が完了
  * identityクレート
  * cryptoクレート
  * earningsクレート
  * subdcriptionクレート
  * notifierクレート
  * contentクレート
  * authクレート
  * repositoryクレート
- phase5のinfraクレート作成も完了。ほとんどのPostgres実装が完了した。共通化関数を使うことでクエリ構文の重複をなくした。
- phase6のcliクレート仮作成も完了。管理者ユーザの作成とmigrationができるようになった。
- migrationができるようになり、shが本番と同じになったため、本運用起動ができるようにした。(nginx.confの設定やDockerfileの修正)

次はphase7のauth-apiをする。
前プロジェクトと違い、すでに本運用もできており、uowの問題も解決しているため、現段階ではでかい問題はないと思っている。

# 2026年07月20日
- phase7の認証系apiを作った

次はphase8

# 2026年07月21日
- phase8の通知系apiを作った。
- phase9のインポート/エクスポートapiを作った。
- phase10の途中までやった

camelCaseとかの問題がめんどい。
動作確認に示されていたものは確認したが、示されていなかった一括設定系はテストしていない。
次はphase10の10節(appクレート: 送信キュー/履歴)。

# 2026年07月22日
- phase10の続きを作った。
- phase11のmonitor/notifyのCLIを作成した

これでバックエンドは9割方完成である。
phase12の残りに進み、本番環境でのテストをする。
そののち、道中にメモをした`指南書からの変更点.md`の変更をしたい点を変えていく。
また、`todo.md`にもいろいろ書いてあるのでそっちも確認する。