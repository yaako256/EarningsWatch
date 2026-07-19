#!/bin/sh
# 本運用時の起動時に実行されるスクリプト

set -e

# migrationを実行
./cli migration

# crontab有効化
echo "start supercronic"
/usr/local/bin/supercronic /app/crontab &

# serverを実行してshをプロセス置換(exec)
exec ./server