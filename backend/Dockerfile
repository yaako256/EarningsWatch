# backend/Dockerfile
# ========================================
# EarningsWatch Backend Dockerfile
# ========================================

# ========================================
# Base Stage (Rust共通)
# ========================================
FROM rust:1.88 AS base

ENV TZ=Asia/Tokyo
WORKDIR /workspace/backend

# sqlxやopenssl系crateで必要になりやすいライブラリを入れる。
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    pkg-config \
    libssl-dev \
    postgresql-client \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# ========================================
# Dev Stage
# ========================================
# 開発用コンテナ。
# cargo-watchとsqlx-cliを入れ、ソースはcomposeでbind mountする。
FROM base AS dev

# Python
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Pythonライブラリ
COPY requirements.txt /tmp/requirements.txt
RUN pip3 install --break-system-packages --no-cache-dir -r /tmp/requirements.txt

# Playwright（※requirementsに含めてない場合のみ）
RUN python3 -m playwright install chromium


# -- lockedを付けて、Rust 1.88環境でビルドできるように
# --version を指定して、Rust 1.88環境で確実にビルドできるバージョン（v0.8.3など）を固定
RUN cargo install cargo-watch --locked \
    && cargo install sqlx-cli --version 0.8.3 --no-default-features --features rustls,postgres --locked

CMD ["cargo", "watch", "-x", "check"]

# ========================================
# Planner Stage
# ========================================
# cargo-chefを使うと依存crateのビルドキャッシュが効きやすい。
# まだプロジェクト初期でも、後で効いてくるので最初から入れておく。
FROM base AS chef
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY backend ./
RUN cargo chef prepare --recipe-path recipe.json

# ========================================
# Builder Stage
# ========================================
FROM chef AS builder

COPY --from=planner /workspace/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# バックエンド全体のコピー
COPY backend ./

RUN cargo build --release -p server -p cli

# ========================================
# Prod Stage
# ========================================
# FROM debian:bookworm-slim AS prod
FROM python:3.12-slim AS prod

ENV TZ=Asia/Tokyo
WORKDIR /app

# 必要ライブラリ
# crulなどに追加して、PlayWright系が多い
RUN apt-get update && apt-get install -y \
    curl \
    tzdata \
    wget \
    ca-certificates \
    libnss3 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxrandr2 \
    libgbm1 \
    libgtk-3-0 \
    libasound2 \
    && rm -rf /var/lib/apt/lists/*


# ----- python -----
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
# Playwright（※requirementsに含めてない場合のみ）
RUN python -m playwright install chromium


# ----- supercronic -----
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
      ARCH="amd64"; \
    elif [ "$ARCH" = "aarch64" ]; then \
      ARCH="arm64"; \
    fi && \
    curl -fsSLO https://github.com/aptible/supercronic/releases/latest/download/supercronic-linux-${ARCH} && \
    mv supercronic-linux-${ARCH} /usr/local/bin/supercronic && \
    chmod +x /usr/local/bin/supercronic


# ----- Rust binary -----
# Rustのrelease binaryだけをコピーする。
COPY --from=builder /workspace/backend/target/release/server /app/server
COPY --from=builder /workspace/backend/target/release/cli /app/cli
COPY config/crontab.prod /app/crontab
COPY scripts/ /app/scripts/

# configはcompose側で /app/config にmountする想定。

COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]


