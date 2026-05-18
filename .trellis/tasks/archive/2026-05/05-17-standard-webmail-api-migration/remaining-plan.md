# Remaining Work Plan (completed 2026-05-18)

## Result

The remaining migration plan from the 2026-05-18 grill-me session has been completed:

- LoginView + AuthContext are wired before app data hooks run.
- Frontend production code uses REST helpers in `api-client.ts` / `api.ts`; no `invoke(...)` path remains.
- `/rpc` and `/rpc/batch` are no longer exposed by the backend or nginx/Vite proxy config.
- Tauri/desktop shell code, desktop package CI, and Tauri build scripts were removed.
- README, deployment config, OpenAPI docs, and Trellis specs were updated for the self-hosted Webmail architecture.

## Final Gate

The completion audit must keep these commands green:

```bash
pnpm test
pnpm run build:frontend
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

## Guardrails Kept

- Frontend inventory tests reject `invoke(`, `tauri-mock`, `/rpc`, and desktop notification contracts.
- CI runs the Webmail quality gate only; it no longer builds Windows/macOS desktop packages.
- Docker/nginx deployment exposes REST/SSE/OAuth/webhook routes only.
