# ==================================================
### Help
# ==================================================
.PHONY: help aaa

## このMakefileのヘルプメッセージを表示
# `###` のコメントをグループ名として表示
# `##` のコメントを各ターゲットの説明として表示
#
# 環境を切り替える場合
#   make help ENV=prod
help:
	@echo ""
	@echo "YaakoDrive Make Commands"
	@echo "Current Environment : $(ENV)"
	@echo ""

		@awk '\
	/^### / { \
		if(group_started){ print "" } \
		group_started=1; \
		printf "\033[1;35m%s\033[0m\n", substr($$0,5); \
		next \
	} \
	/^## / { \
		desc=substr($$0,4) \
	} \
	/^[a-zA-Z0-9_.-]+:/ { \
		if(desc){ \
			sub(/:.*/,"",$$1); \
			printf "  \033[36m%-22s\033[0m %s\n",$$1,desc; \
			desc="" \
		} \
	} \
	END { \
		if(group_started){ print "" } \
	}' $(MAKEFILE_LIST)


# ==================================================
### おもしろコマンド
# ==================================================
.PHONY:	progressbar git-now matrix coffee
## ただのプログレスバーを表示
progressbar:
	@printf "Loading commands "
	@for i in 1 2 3 4 5 6 7 8 9 10; do \
		printf "█"; \
		sleep 0.5; \
	done
	@echo " done"

## gitのブランチ名と現在時刻を表示
git-now:
	@echo "Branch : $$(git branch --show-current)"
	@echo "Time   : $$(date '+%Y-%m-%d %H:%M:%S')"

## ターミナルをハッカー風にする
matrix:
	@for i in $$(seq 1 30); do \
		echo "\033[0;32m$$(tr -dc A-Za-z0-9 </dev/urandom | head -c 80)\033[0m"; \
		sleep 0.1; \
	done

# コーヒーパワー注入
coffee:
	@echo "☕ Brewing coffee..."
	@sleep 1
	@echo "██░░░░░░░░ 20%"
	@sleep 1
	@echo "█████░░░░░ 50%"
	@sleep 1
	@echo "██████████ 100%"
	@echo "☕ Coffee ready. Coding power +10"
