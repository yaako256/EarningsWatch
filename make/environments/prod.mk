# ==================================================
# 環境固有定数(prod)
# ==================================================
COMPOSE := docker compose -f compose.yaml -f compose.prod.yaml
DB_NAME := earningswatch_prod
CLI := /app/yaakodrive-cli # todo!実装時に変更
CONTAINER_SHELL := sh
MIGRATIONS_PATH := /app/sql/migrations # todo!実装時に変更


# ==================================================
### 環境固有コマンド(prod)
# ==================================================
.PHONY:	setup	deploy

## 本番環境初回セットアップ
#
# 初回のみ実行する。
#
# 作成ディレクトリ
# /srv/yaakodrive/postgres
# /srv/yaakodrive/data/files
# /srv/yaakodrive/data/tmp
#
setup:
	sudo mkdir -p /srv/yaakodrive/postgres
	sudo mkdir -p /srv/yaakodrive/data/files
	sudo mkdir -p /srv/yaakodrive/data/tmp
	@echo "本番用ディレクトリを作成しました"


## 本番デプロイ
# make deploy ENV=prod
deploy:
	$(COMPOSE) up -d --build --force-recreate
