#!/bin/sh
# 本運用時の起動時に実行されるスクリプト

set -e

# Phase 6でCLI(create-admin, migration)実装完了後にコメントアウトを解除する
# (design/00-overview.md 7.1章)
# ./cli migration

# crontab有効化
echo "start supercronic"
exec /usr/local/bin/supercronic /app/crontab


exec ./server