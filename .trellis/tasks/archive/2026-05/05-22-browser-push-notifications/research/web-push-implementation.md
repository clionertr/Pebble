# Web Push Implementation Research

## Sources

- MDN Push API: service worker receives push events when the app is not foreground or loaded; subscription contains endpoint and encryption keys; endpoint is a capability URL and must be protected.
- MDN Notifications API: notification permission should be requested from a user gesture; persistent notifications are created from a service worker and handled by `notificationclick`.
- MDN `ServiceWorkerRegistration.showNotification()`: secure context required; supports title, body, data, tag, icon, timestamp and click handling.
- `web-push` Rust crate docs: crate sends encrypted Web Push payloads with RFC8188 and VAPID; `SubscriptionInfo`, `WebPushMessageBuilder`, `VapidSignatureBuilder`, `ContentEncoding::Aes128Gcm`, and an async client are the main API.

## Constraints From Pebble

- Pebble is a self-hosted single-user webmail app with cookie authentication and in-memory sessions.
- The backend already emits `mail:new` through SSE, but closed pages need Web Push instead of SSE.
- Existing notification preference is a placeholder and defaults to enabled; real browser notifications must default to off for new devices.
- Existing sync workers already provide `StoredMessage.notify`, which can avoid notifying for initial historical sync/refresh paths.
- Existing rule engine runs in `server/src/rpc/indexing.rs`; notification eligibility must be checked after rule actions change final folder membership.
- Production deployment serves the frontend through nginx and proxies `/api`, `/events`, `/auth`, and `/webhook` to the backend. Service worker files must be served as static frontend assets.

## Recommended Approach

- Use the Rust `web-push` crate rather than hand-rolling encryption and VAPID signing.
- Add a `public/pebble-sw.js` service worker so Vite copies it to the web root and browsers can register it under `/pebble-sw.js` with root scope.
- Add backend REST endpoints under `/api/notifications/*` for VAPID public key, subscription upsert/delete, device list/update/delete, and test notification.
- Persist subscriptions in SQLite and remove or mark invalid subscriptions when push sends return permanent endpoint errors.
- Keep notification payloads small and explicit: title, body, URL/action target, type, message id when applicable, tag, timestamp.
- Use `ServiceWorkerRegistration.showNotification()` inside the push handler and `clients.openWindow()`/`clients.matchAll()` in `notificationclick`.

## Implementation Risks

- Web Push and notification APIs require secure context except localhost/dev exceptions.
- Browser permission cannot be requested automatically; old enabled preferences can only auto-restore when permission is already `granted`.
- Service worker tests in jsdom are limited; most frontend behavior should be tested around pure helpers and API calls, with manual/browser verification noted.
- VAPID key generation/persistence needs careful format handling because the browser subscription call needs a base64url public key while the Rust sender needs private signing material.
