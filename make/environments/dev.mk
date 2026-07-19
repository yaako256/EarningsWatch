# make/environments/dev.mk
# ==================================================
# 環境固有定数(dev)
# ==================================================
COMPOSE := docker compose -f compose.yaml -f compose.dev.yaml
DB_NAME := earningswatch_dev
CLI := cargo run -p cli --
CONTAINER_SHELL := bash
MIGRATIONS_PATH := /workspace/sql/migrations # todo!実装時に変更

# ==================================================
### 環境固有コマンド(dev)
# ==================================================
.PHONY: run-server check test dev-reset npm-install

## サーバ起動（開発用）
run-server:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		cargo run -p server

## Cargo check
check:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		cargo check

## Cargo test
test:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		cargo test

## 開発環境を完全リセット
# DB・コンテナを再作成し、migrationまで実行する
dev-reset:
	$(COMPOSE) down -v
	$(COMPOSE) up -d --build
	$(MAKE) migrate

## 完全リセット後の必須処理(npm install)
# これを実行したのち、make upする
npm-install:
	$(COMPOSE) run --rm $(FRONTEND_SERVICE_NAME) npm install



# ----------------------------------
# SQL系統
# ----------------------------------
## SQLファイル作成
# 実行例: make migrate-add NAME=create_users
migrate-add:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		sqlx migrate add --source $(MIGRATIONS_PATH) $(NAME)

## PostgreSQLへ接続
migrate-psql:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		bash -c 'psql $$DATABASE_URL'

## DB一覧表示
migrate-psql-db:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		bash -c 'psql $$DATABASE_URL -c "\l"'

## migrationリセット
migrate-reset:
	$(COMPOSE) down
	docker volume rm yaakodrive-dev_postgres_data_dev || true
	$(COMPOSE) up -d
	@echo "Waiting for database..."
	$(MAKE) migrate

## sqlx prepare
sqlx-prepare:
	$(COMPOSE) exec $(BACKEND_SERVICE_NAME) \
		cargo sqlx prepare --workspace