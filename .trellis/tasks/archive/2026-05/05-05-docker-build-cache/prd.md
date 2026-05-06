# PRD: Docker Build Cache Optimization

## 1. Background
The current Docker build process lacks efficient caching. The Rust backend re-compiles all dependencies whenever any source file changes, and the frontend re-downloads pnpm packages unnecessarily. CI workflows also lack Docker image publication and caching.

## 2. Goals
- Minimize local Docker build times using BuildKit mounts.
- Speed up CI Docker builds using GitHub Actions cache (`type=gha`).
- Support multi-architecture images (`linux/amd64`, `linux/arm64`).
- Automate image publication to GitHub Packages (ghcr.io).

## 3. Technical Requirements

### 3.1 Backend (Rust)
- Use `cargo-chef` to pre-build dependencies in a separate layer.
- Use BuildKit cache mounts (`--mount=type=cache`) for `target/` and `/usr/local/cargo/registry` to support incremental compilation.
- Multi-stage build to keep the final image slim (using `debian:bookworm-slim`).

### 3.2 Frontend (React)
- Use pnpm with a persistent store mount (`--mount=type=cache`).
- Multi-stage build with Nginx as the final runtime.

### 3.3 CI/CD (GitHub Actions)
- Integrate `docker/setup-qemu-action` and `docker/setup-buildx-action`.
- Implement `docker/build-push-action` in `release.yml` (and potentially a separate Docker CI workflow).
- Configure `cache-from: type=gha` and `cache-to: type=gha,mode=max`.
- Tagging strategy:
    - Master push: `ghcr.io/user/pebble:edge`
    - Tag push: `ghcr.io/user/pebble:v*.*.*` and `ghcr.io/user/pebble:latest`

## 4. Completion Criteria
- [ ] `backend.Dockerfile` optimized with `cargo-chef` and mounts.
- [ ] `frontend.Dockerfile` optimized with pnpm store mounts.
- [ ] GitHub Actions updated to build, cache, and push multi-arch images.
- [ ] Successful local build verification.
- [ ] Successful CI build verification (simulated or via workflow files).
