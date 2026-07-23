# ==================================================
# 環境固有定数(prod)
# ==================================================
COMPOSE := docker compose -f compose.yaml -f compose.prod.yaml
DB_NAME := earningswatch_prod
CLI := /app/cli # todo!実装時に変更
CONTAINER_SHELL := sh
MIGRATIONS_PATH := /app/sql/migrations # todo!実装時に変更


# ==================================================
### 環境固有コマンド(prod)
# ==================================================
.PHONY:	setup	deploy

## 本番デプロイ
# make deploy ENV=prod
deploy:
	$(COMPOSE) up -d --build --force-recreate
