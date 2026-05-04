# PRD: Fix Backend Binding Host for Docker Deployment

## Problem
The backend service (Pebble) is hardcoded to listen on `127.0.0.1:3000` in `src-tauri/src/main.rs`. When deployed via Docker Compose, the frontend container (or a reverse proxy on another network interface) cannot reach the backend because it is only listening on the loopback interface inside the backend container.

## Requirements
- Allow the backend to listen on an arbitrary host and port.
- Use environment variables `PEBBLE_HOST` and `PEBBLE_PORT` to configure the binding address.
- Default to `127.0.0.1` and `3000` for backward compatibility with local development/Tauri.
- In Docker environments, we can then set `PEBBLE_HOST=0.0.0.0` to allow external connections.

## Acceptance Criteria
- Code modified to read environment variables for host and port.
- Default values remain `127.0.0.1` and `3000`.
- Verified that the backend starts and logs the correct listening address.
