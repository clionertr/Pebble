FROM node:22-alpine AS builder
WORKDIR /app
ARG TARGETARCH

# Install pnpm using corepack
RUN corepack enable && corepack prepare pnpm@11.1.1 --activate

# Copy package files
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# Install dependencies with cache mount for pnpm store
RUN --mount=type=cache,id=pnpm-${TARGETARCH},target=/pnpm/store,sharing=locked \
    PNPM_HOME="/pnpm" pnpm install --frozen-lockfile

# Cache-busting arg: change to force rebuild from this point
ARG CACHEBUST=0

# Copy source code and build frontend
COPY . .
RUN echo "Build: ${CACHEBUST}" && pnpm run build:frontend

# Runtime stage: Nginx
FROM nginx:alpine

# Remove default nginx static assets
RUN rm -rf /usr/share/nginx/html/*

# Copy built frontend assets
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom nginx configuration for routing and reverse proxy
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
