# make/cli.mk
# ==================================================
### CLI
# ==================================================
.PHONY:	cli

## 任意のCLIコマンドを実行
#
# 実行例:
# make cli ARGS="migrate"
# make cli ARGS="create-admin --username yaako"
# make cli ENV=prod ARGS="create-admin --username yaako"
cli:
	$(BACKEND_EXEC) $(CLI) $(ARGS)





# 管理者ユーザ作成(いらない説)(古い)
#
# Usage
#
# make create-admin USERNAME=yaako
#
# 本番環境
#
# make create-admin USERNAME=yaako ENV=prod
#
# create-admin:
# 	$(MAKE) cli \
# 		ENV=$(ENV) \
# 		ARGS="create-admin --username $(USERNAME)"