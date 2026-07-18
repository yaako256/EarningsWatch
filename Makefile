# Makefile
# メモ => PHONY: ファイルではないという指定(ファイルは更新されていないと実行されない): 命令である



# ==================================
### その他 (Utilities)
# ==================================
.PHONY: chown tree help

## カレントディレクトリ内の全ファイルに権限の付与
chown:
	sudo chown -R $(shell whoami):$(shell whoami) .

## フォルダツリーを表示 (自作Pythonスクリプト実行)
tree:
	python3 ./generate_tree_ver2.py . 100 .git

## このMakefileのヘルプメッセージを表示
# `#`が3つのものを検知し、グループ名を表示している
# `#`が2つのものを検知し、そのあとのkeyと組み合わせることでhelpを表示している
help:
	@awk '/^### / {print ""; printf "\033[1;35m%s\033[0m\n", substr($$0, 5); next} /^## / {desc=substr($$0, 4)} /^[a-zA-Z_-]+:/ {if (desc) {sub(/:.*/, "", $$1); printf "  \033[36m%-15s\033[0m %s\n", $$1, desc; desc=""}}' $(MAKEFILE_LIST)

