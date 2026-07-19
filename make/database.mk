# make/database.mk
# ==================================================
### Database
# ==================================================

.PHONY: \
	migrate \
	user user-x \
	token token-x \
	node node-x \
	file file-x

# ==================================================
# Migration
# ==================================================

## migrationの実行
# dev:
#   cargo run -p cli -- migrate
# prod:
#   /app/yaakodrive-cli migrate
# migrate:
# 	$(MAKE) cli \
# 		ARGS="migrate --migrations-path $(MIGRATIONS_PATH)"

# ==================================================
# Database Viewer
# ==================================================
## usersテーブル(一部)
db-users:
	$(PSQL) -c "$(SQL_USER)"

## usersテーブル(全件)
db-users-x:
	$(PSQL) -x -c "SELECT * FROM users;"

## refresh_tokensテーブル(一部)
db-tokens:
	$(PSQL) -c "$(SQL_TOKEN)"

## refresh_tokensテーブル(全件)
db-tokens-x:
	$(PSQL) -x -c "SELECT * FROM refresh_tokens;"

## nodesテーブル(一部)
db-nodes:
	$(PSQL) -c "$(SQL_NODE)"

## nodesテーブル(全件)
db-nodes-x:
	$(PSQL) -x -c "SELECT * FROM nodes;"

## file_contentsテーブル(一部)
db-files:
	$(PSQL) -c "$(SQL_FILE)"

## file_contentsテーブル(全件)
db-files-x:
	$(PSQL) -x -c "SELECT * FROM file_contents;"