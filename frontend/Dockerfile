# frontend/Dockerfile
# ========================================
# EarningsWatch Frontend Dockerfile
# ========================================

# ========================================
# Base Stage (Node.js共通)
# ========================================
FROM node:22-bookworm AS base

WORKDIR /workspace/frontend

# ========================================
# Dev Stage
# ========================================
FROM base AS dev

#CMD ["sleep", "infinity"]
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]

# ========================================
# Builder Stage
# ========================================
FROM base AS builder

COPY frontend/package*.json ./

# builder は lockfile を信用して再現可能ビルドしたいので ci
RUN npm ci

COPY frontend ./

RUN npm run build

# ========================================
# Prod Stage
# ========================================
FROM nginx:1.27-alpine AS prod

COPY --from=builder /workspace/frontend/dist /usr/share/nginx/html
# nginx設定ファイルをコピー
COPY frontend/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
