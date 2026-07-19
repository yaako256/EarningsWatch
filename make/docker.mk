# make/docker.mk
# ==================================================
### Docker
# ==================================================
.PHONY: up down stop build recreate restart logs ps

## コンテナ起動
# buildも実施する
up:
	$(COMPOSE) up --build

## コンテナ停止
stop:
	$(COMPOSE) stop

## コンテナ停止・削除
down:
	$(COMPOSE) down

## Dockerイメージをbuild
build:
	$(COMPOSE) build

## コンテナ再作成（env変更反映）
recreate:
	$(COMPOSE) up -d --force-recreate

## コンテナ再起動
restart:
	$(COMPOSE) down
	$(COMPOSE) up

## ログ表示
# 今後サービス指定をできるようにしてもいいかも
logs:
	$(COMPOSE) logs -f

## コンテナ一覧
ps:
	$(COMPOSE) ps


.PHONY: backend-shell frontend-shell

## Backendコンテナへ入る
backend-shell:
	$(BACKEND_EXEC) $(CONTAINER_SHELL)

## Frontendコンテナへ入る
frontend-shell:
	$(FRONTEND_EXEC) $(CONTAINER_SHELL)