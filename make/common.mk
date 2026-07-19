# make/common.mk
# ==================================================
# 共通設定
# ==================================================
# デフォルトコマンド
.DEFAULT_GOAL := help

# デフォルト環境
ENV ?= dev

# ----------------------------------
# 共通定数定義
# ----------------------------------
# サービス名
BACKEND_SERVICE_NAME := backend
FRONTEND_SERVICE_NAME := frontend
DATABASE_SERVICE_NAME := db

# DBユーザ名
DB_USER := earningswatch

# ----------------------------------
# 環境ごとの定数やコマンドのロード
# ----------------------------------
# 環境のenum
SUPPORTED_ENVS = dev prod
ifeq ($(filter $(ENV),$(SUPPORTED_ENVS)),)
$(error Invalid ENV=$(ENV). Use ENV=dev or ENV=prod)
endif

include make/environments/$(ENV).mk

# ----------------------------------
# 共通コマンドの定義
# ----------------------------------
BACKEND_EXEC = $(COMPOSE) exec $(BACKEND_SERVICE_NAME)
FRONTEND_EXEC = $(COMPOSE) exec $(FRONTEND_SERVICE_NAME)
DB_EXEC = $(COMPOSE) exec $(DATABASE_SERVICE_NAME)
CLI_EXEC = $(BACKEND_EXEC) $(CLI)
SQLX = $(BACKEND_EXEC) sqlx
PSQL = $(DB_EXEC) psql -U $(DB_USER) -d $(DB_NAME)

# ----------------------------------
# SQL定義
# 通常表示で表示する項目を管理
# この定義は別ファイルに移してもいいかも
# ----------------------------------
# todo!実装時に本プロジェクト用に変更
SQL_USER = \
	SELECT id, username, role, password_hash \
	FROM users;

SQL_TOKEN = \
	SELECT user_id, user_agent, created_at, revoked_at \
	FROM refresh_tokens;

SQL_NODE = \
	SELECT id, owner_user_id, parent_id, name, node_type, updated_at, deleted_at \
	FROM nodes;

SQL_FILE = \
	SELECT node_id, stored_filename, mime_type, size_bytes, status \
	FROM file_contents;