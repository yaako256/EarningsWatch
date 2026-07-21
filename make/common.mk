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
