# make/utility.mk
# ==================================================
### Developer utilities
# ==================================================
.PHONY:	tree chown

## カレントディレクトリ以下のファイルに権限を付与
chown:
	sudo chown -R $(shell whoami):$(shell whoami) .

## フォルダツリーを表示(自作Pythonスクリプト)
tree:
	python3 ./generate_tree_ver2.py . 100 target .git .sqlx frontend
