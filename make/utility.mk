# make/utility.mk
# ==================================================
### Developer utilities
# ==================================================
.PHONY:	tree chown ls

## カレントディレクトリ以下のファイルに権限を付与
chown:
	sudo chown -R $(shell whoami):$(shell whoami) .

## フォルダツリーを表示(自作Pythonスクリプト)
tree:
	python3 ./generate_tree_ver2.py . 100 target .git .sqlx frontend

## 稼働中のdocker compose一覧
ls:
	docker compose ls -a

## 停止中も含めたdocker compose一覧
ls-a:
	docker compose ls -a

# その他compose一覧表示方法
# 停止中も含めて: docker compose ls -a
# json形式で: docker compose ls --format json | jq