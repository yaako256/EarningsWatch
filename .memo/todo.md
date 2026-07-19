## todo

設計書が完成した。
今は開発指南書を作らせている。

これが出来たらついにPhase 0から取り掛かっていく。

一旦コミでコミットして、ブランチ作って、環境作りからやっていこう。


# requirements.txt
requirements.txtを作らなきゃいけん
後で作る。
現在はrequirements.txtをYutaiWatchからコピってくることで解決しており、devステージでもrequirements.txtを使っている。

# Makefile群
SQLテーブルや、cliコマンド、migrationパス(そもそもいらないかも)などがYaakoDriveのままとなっている。
実際に使うときになったら編集する。

# composeのport
今後おかしくなる可能性がある。
YaakoDrive含め、ポートの整理をした方がいいかもしれない。
開発用ポートを何番にするとか、10単位でプロジェクトを分けるとか。

# ログ何件溜まったら
configへ切り出す