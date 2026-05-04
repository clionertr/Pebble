# Research: 502 Bad Gateway in 1Panel Docker Compose

## Findings
1. **Error**: User reported 502 error when accessing the site via 1Panel reverse proxy.
2. **Analysis**: 
   - Frontend logs showed `connect() failed (111: Connection refused) while connecting to upstream http://backend:3000`.
   - Backend process is running.
   - Checking `/proc/net/tcp` inside the backend container revealed it was listening on `127.0.0.1:3000` (hex `0100007F:0BB8`).
3. **Root Cause**: The backend service is hardcoded to `127.0.0.1` in `src-tauri/src/main.rs`.
4. **Fix**: Change the binding address to be configurable via environment variables, and set it to `0.0.0.0` in Docker environments.
