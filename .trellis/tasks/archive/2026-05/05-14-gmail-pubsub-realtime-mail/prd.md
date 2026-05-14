# brainstorm: Gmail Pub/Sub realtime mail

## Goal

Add Gmail near-realtime mail intake to Pebble Webmail using Gmail API `watch` notifications delivered through Google Cloud Pub/Sub, so Gmail accounts can receive new-mail updates without relying only on periodic polling.

## What I Already Know

* The desired path is Gmail API + Cloud Pub/Sub, not IMAP IDLE.
* Gmail publishes mailbox change notifications to a Pub/Sub topic after each account calls `users.watch`.
* The user supplied a public push endpoint on `pebble.ailolis.net` with a shared-secret query parameter. The exact secret is intentionally not persisted in this PRD.
* Push handling must acknowledge Google quickly and perform Gmail fetch/sync work asynchronously.
* `watch` renewal, OAuth access-token refresh, `historyId` handling, and fetching changed messages are in scope for design.
* Pebble is now a self-hosted web service: Axum backend, React frontend, `/events` SSE, and JSON-RPC under `/rpc`.
* Existing Gmail sync already uses Gmail history deltas and stores the cursor in account `sync_state`.
* Existing realtime plumbing already supports a provider-push trigger concept.

## Assumptions (Temporary)

* The feature should integrate with existing Pebble account storage and sync logic instead of introducing a separate Gmail ingestion model.
* The webhook receiver may need to be a deployable server process rather than only the local Tauri desktop backend, because Pub/Sub push requires a public HTTPS endpoint.
* The existing UI realtime status should reflect Gmail Pub/Sub health where practical.

## Open Questions

* None at this point; requirements are ready to converge into MVP scope.

## Decisions

* The existing Pebble backend process will own the public `/webhook/gmail` endpoint.
* The webhook handler will use the existing local encrypted OAuth token store and existing Gmail sync path instead of introducing a separate ingestion service.
* Pub/Sub notifications will be translated into existing provider-push sync triggers for the matching Gmail account.
* MVP webhook authentication will use the existing URL shared secret only.
* Pub/Sub OIDC JWT verification is deferred to a later security-hardening pass and remains required before treating the endpoint as production-hardened.
* Gmail Pub/Sub `watch` registration must be user-initiated via an "Enable realtime Gmail" action, not automatic during OAuth login.
* Existing Gmail OAuth login, IMAP account login, and existing IMAP realtime/polling behavior must remain unchanged.
* Gmail realtime push is enabled per Gmail account, not globally for all Gmail accounts.
* Gmail push-enabled accounts should use a push-first strategy with low-frequency polling as a fallback.
* Push-enabled Gmail accounts use a configurable fallback polling interval that does not follow the existing Realtime/Balanced/Battery intervals unless explicitly decided.
* Global Manual-only mode still stops background fallback polling, but an incoming Pub/Sub push may trigger a one-shot sync for the matching account.
* Gmail push/watch state will be stored in the existing per-account `sync_state.extra.gmail_push` JSON field, not in a new database table for MVP.
* Expected Gmail push state fields: `enabled`, `topic_name`, `expiration_ms`, `last_watch_history_id`, `last_watch_at`, `last_error`, and `fallback_interval_minutes`.
* Webhook notifications map only to accounts where `provider=gmail`, `email` exactly matches the decoded Gmail `emailAddress`, and `sync_state.extra.gmail_push.enabled=true`.
* If no local account matches a valid push notification, the webhook should return 200 and log the unmapped email rather than force Pub/Sub retries.
* If multiple local Gmail accounts match, all push-enabled matches should receive a provider-push sync trigger.
* Gmail Pub/Sub topic and webhook shared secret are runtime configuration values:
  * `GMAIL_PUBSUB_TOPIC=projects/<project-id>/topics/<topic-name>`
  * `GMAIL_WEBHOOK_SECRET=<secret>`
* Missing Gmail Pub/Sub configuration should block only the explicit Gmail realtime enable action and webhook validation, not OAuth login or normal mail sync.
* MVP includes both per-account enable and disable actions for Gmail realtime.
* Disabling Gmail realtime should call Gmail `users.stop` for that account when possible, clear or mark `sync_state.extra.gmail_push.enabled=false`, and return the account to normal Gmail polling behavior.
* Webhook response rules:
  * Missing or incorrect `secret`: return 401 and do not trigger sync.
  * Malformed JSON, invalid base64 data, or missing required decoded fields: return 400 and do not trigger sync.
  * Valid payload with no matching push-enabled local account: return 200 and log.
  * Valid payload that triggers or queues sync: return 200 immediately; later sync failures do not affect Pub/Sub acknowledgement.
* Watch renewal strategy:
  * On backend startup, scan Gmail accounts with `sync_state.extra.gmail_push.enabled=true`.
  * Renew `watch` when `expiration_ms` is missing or less than 24 hours away.
  * Run the same renewal check every 12 hours while the backend is running.
* Failure handling:
  * If user-initiated enable fails, return a visible error, do not set `gmail_push.enabled=true`, and leave normal Gmail sync unchanged.
  * If background renewal fails, keep `gmail_push.enabled=true`, persist `last_error`, show an error/retry status in the UI, keep fallback polling enabled according to the configured interval, and retry on the next renewal cycle.
  * If enable/renewal fails because OAuth refresh or Gmail auth fails, surface the account as reconnect-required.
* Gmail `watch` scope is Inbox-only for MVP, using Gmail label `INBOX`.
* Gmail push fallback polling interval should be configurable instead of fixed at 15 minutes.
* Gmail push fallback polling interval is configured per Gmail account, defaulting to 15 minutes.
* Allowed fallback polling interval range is 1 to 60 minutes.
* Gmail realtime UI controls live under Settings -> Accounts:
  * The Gmail account row shows current Gmail realtime state plus enable/disable action.
  * The account edit modal exposes the per-account fallback interval control.
* The global realtime mode UI should not become the Gmail push configuration surface.
* Gmail realtime UI status labels should cover: `Not enabled`, `Enabling...`, `Realtime enabled`, `Renewing...`, `Realtime error`, `Reconnect required`, and `Config missing`.
* Backend JSON-RPC methods for Gmail realtime controls:
  * `get_gmail_realtime_config(account_id)`
  * `enable_gmail_realtime(account_id, fallback_interval_minutes)`
  * `disable_gmail_realtime(account_id)`
  * `update_gmail_realtime_config(account_id, fallback_interval_minutes)`
* After `enable_gmail_realtime` successfully calls Gmail `watch`, trigger an immediate sync for that account.
* Store the `historyId` returned by `watch` as `gmail_push.last_watch_history_id`; do not overwrite the existing Gmail sync cursor with it.
* Pub/Sub push handling should reuse `trigger_sync(account_id, "provider_push")` behavior so an account sync can run even if no normal sync worker is currently active.
* Pub/Sub push triggers should be lightly coalesced per account: if a push-triggered sync is already queued or running within a 30-second window, do not enqueue duplicate sync work for that account.
* Coalescing may record the most recent push `historyId`/timestamp for diagnostics, but Gmail History sync correctness must continue to rely on the stored Gmail sync cursor.
* MVP includes deployment documentation for:
  * GCP Gmail API and Pub/Sub API enablement.
  * Pub/Sub topic creation.
  * Granting Pub/Sub Publisher to `gmail-api-push@system.gserviceaccount.com`.
  * Push subscription pointing at `/webhook/gmail?secret=<secret>`.
  * Reverse proxy exposure for `/webhook/gmail`.
  * `GMAIL_PUBSUB_TOPIC` and `GMAIL_WEBHOOK_SECRET` environment variables.
* Keep the existing Gmail OAuth scope (`https://mail.google.com/`) for this task.
* Do not narrow Gmail OAuth to readonly because existing Pebble Gmail features include send, label/flag changes, delete/archive, and drafts.

## Requirements (Evolving)

* Register or renew Gmail `watch` for connected Gmail accounts.
* Receive Pub/Sub push notifications for Gmail mailbox changes.
* Decode Pub/Sub messages and map Gmail `emailAddress` to a Pebble account.
* Use Gmail `history.list` / message fetch flow to import changed messages into the existing local store.
* Avoid blocking Pub/Sub push acknowledgement on long-running sync work.
* Handle expired access tokens and expired Gmail watch subscriptions.
* Run inside the existing Pebble backend deployment and write to the same SQLite/store/index/event pipeline as normal sync.
* Keep the webhook shared secret out of source control and load it from runtime configuration.
* Add an explicit UI/API action for enabling Gmail realtime push without changing account creation/login flow.
* Store Gmail push/watch state per account.
* Persist Gmail watch metadata under `sync_state.extra.gmail_push` and preserve sibling sync state fields.
* Add account lookup logic for Gmail push notifications that filters by provider, email, and per-account push-enabled state.
* Add an explicit per-account disable action for Gmail realtime push.
* Validate and reject malformed webhook requests before account lookup or sync trigger.
* Keep a fallback poll path for push-enabled Gmail accounts to recover from missed Pub/Sub notifications, watch expiry, and service restarts.
* Preserve the existing global realtime preference behavior for IMAP and non-push-enabled accounts.
* Update deployment docs for Gmail Pub/Sub setup and webhook routing.
* Preserve existing Gmail OAuth scopes and account-login behavior.
* Validate per-account Gmail push fallback interval as 1-60 minutes, defaulting to 15.
* Add per-account Gmail realtime controls in Settings -> Accounts without changing global realtime preference controls.
* Expose enough backend state for the UI to render the agreed Gmail realtime statuses.
* Add JSON-RPC handlers for reading, enabling, disabling, and updating per-account Gmail realtime settings.
* Trigger a sync after successful Gmail watch enablement without skipping prior unsynced history.
* Trigger Gmail sync from webhook notifications even when the account's regular sync worker is not already running.
* Avoid duplicate sync storms from bursty Pub/Sub notifications by coalescing account triggers.

## Acceptance Criteria (Evolving)

* [ ] A Gmail account can be subscribed to Gmail push notifications.
* [ ] A valid Pub/Sub push notification causes new Gmail messages to appear through the existing mailbox UI/update path.
* [ ] Push acknowledgement returns promptly even when message fetch work continues in the background.
* [ ] Watch renewal is scheduled before Gmail watch expiration.
* [ ] Backend startup renews enabled Gmail watches that are missing expiration data or expiring within 24 hours.
* [ ] A failed user-initiated enable does not mark the account as push-enabled.
* [ ] A failed background renewal records an account-level error while preserving fallback sync.
* [ ] OAuth/auth failures surface as reconnect-required.
* [ ] Token refresh is used when Gmail API calls encounter expired access tokens.
* [ ] Invalid webhook requests are rejected or ignored without triggering sync.
* [ ] Requests with a missing or incorrect webhook secret do not trigger sync.
* [ ] OAuth Gmail login still succeeds without requiring Pub/Sub configuration.
* [ ] IMAP accounts continue using the existing IMAP path and are not routed through Gmail Pub/Sub code.
* [ ] Enabling Gmail realtime for one Gmail account does not enable it for other Gmail accounts.
* [ ] A push-enabled Gmail account still performs configurable low-frequency fallback sync even if no Pub/Sub notification arrives.
* [ ] When global Manual-only is selected, push-enabled Gmail accounts do not run periodic fallback polling but can still sync in response to a valid Pub/Sub notification.
* [ ] Enabling Gmail realtime updates only the target account's `sync_state.extra.gmail_push` metadata.
* [ ] Disabling Gmail realtime calls Gmail `users.stop` when possible and prevents future Pub/Sub notifications for that account from triggering sync locally.
* [ ] Each push-enabled Gmail account can have its own fallback polling interval.
* [ ] Fallback interval validation accepts 1-60 minutes and defaults to 15 minutes.
* [ ] Gmail account rows can enable or disable Gmail realtime push.
* [ ] Gmail account edit UI can configure that account's fallback interval.
* [ ] UI can distinguish not-enabled, in-progress, enabled, renewal, error, reconnect-required, and missing-config states.
* [ ] The frontend can read and update Gmail realtime settings through dedicated JSON-RPC methods.
* [ ] Enabling Gmail realtime triggers an immediate sync.
* [ ] Watch response `historyId` does not overwrite the account's existing sync cursor.
* [ ] A valid Pub/Sub push for a push-enabled account starts a sync even if no worker was already running.
* [ ] Burst Pub/Sub notifications for the same account are coalesced within 30 seconds.
* [ ] Documentation explains GCP setup, reverse proxy routing, and required env vars.
* [ ] Existing Gmail send/modify/draft behavior remains authorized by the current OAuth scope.
* [ ] A valid notification for an unknown Gmail address returns 200 without triggering sync.
* [ ] Multiple matching push-enabled Gmail accounts are all triggered.
* [ ] Bad webhook secret returns 401.
* [ ] Malformed Pub/Sub payload returns 400.

## Definition of Done

* Tests added or updated for new backend logic and affected UI status behavior.
* Rust lint/typecheck/tests and frontend tests pass where relevant.
* Sensitive deployment secrets are not committed.
* Production-hardening follow-up for Pub/Sub OIDC JWT verification is documented if not implemented in this task.
* Rollout/rollback and external GCP setup requirements are documented.

## Out of Scope (Explicit)

* Replacing non-Gmail realtime behavior.
* Full Google Cloud infrastructure provisioning automation unless explicitly agreed.
* Storing email content in any server-side service unless agreed as part of the deployment model.

## Technical Notes

* Likely impacted areas: `src-tauri/src/main.rs`, new or existing server module for webhook/watch scheduling, `src-tauri/src/rpc/oauth.rs`, `src-tauri/src/rpc/sync_cmd.rs`, `crates/pebble-mail/src/provider/gmail.rs`, `crates/pebble-mail/src/gmail_sync.rs`, `crates/pebble-store/src/accounts.rs`, and settings/status UI tests.
* `src-tauri/src/main.rs` currently registers `/rpc`, `/rpc/batch`, `/events`, `/auth/login`, and `/auth/callback`; no webhook route exists yet.
* `src-tauri/src/auth.rs` creates or updates OAuth accounts and persists encrypted OAuth token data.
* `src-tauri/src/rpc/oauth.rs` already has token refresh helpers and encrypted token read/write helpers.
* `crates/pebble-mail/src/gmail_sync.rs` already implements `poll_changes()` using Gmail History API and message fetch/storage/event emission.
* `crates/pebble-mail/src/realtime_policy.rs` already includes `SyncTrigger::ProviderPush`.
* `src-tauri/src/rpc/sync_cmd.rs` has `trigger_sync(account_id, "provider_push")`, which can wake an existing sync worker or start a one-shot sync.
* Existing deploy docs proxy `/rpc`, `/events`, and `/auth`; `/webhook/gmail` would need reverse-proxy exposure.
* User confirmed the existing Pebble backend should directly receive Pub/Sub push and own token-backed sync work.
* User chose a staged authentication rollout: URL secret for MVP, Pub/Sub OIDC JWT later.
* User wants Gmail realtime push enabled only when the user clicks an explicit control, preserving IMAP and existing login flows.
* User chose per-account Gmail realtime enablement.
* User chose push-first Gmail sync with low-frequency polling fallback.
* User initially confirmed a fixed 15-minute fallback interval, then revised it to be configurable.
* User confirmed Gmail push/watch state should live in `sync_state.extra.gmail_push` instead of a new table.
* User confirmed the webhook matching strategy: trigger only push-enabled matching Gmail accounts, ignore unknown accounts with a 200 response, and trigger all matches if duplicated.
* User confirmed `GMAIL_PUBSUB_TOPIC` and `GMAIL_WEBHOOK_SECRET` runtime env vars for topic and secret configuration.
* User confirmed per-account disable support should be in MVP.
* User confirmed webhook status behavior for bad secrets, malformed payloads, unknown accounts, and successful trigger acknowledgement.
* User confirmed startup plus 12-hour renewal checks, renewing watches with missing or less-than-24-hour expiration.
* User confirmed failure handling for enable, renewal, fallback polling, and OAuth reconnect-required status.
* User continued with Inbox-only watch scope from the previous proposed option.
* User chose per-Gmail-account fallback interval configuration.
* User set the fallback interval default to 15 minutes and allowed range to 1-60 minutes.
* User confirmed Gmail realtime controls belong in Settings -> Accounts, with row-level enable/disable and edit-modal fallback interval configuration.
* User confirmed the minimum UI status set for Gmail realtime.
* User confirmed the backend JSON-RPC method boundaries for Gmail realtime controls.
* User confirmed enablement should trigger immediate sync and should not overwrite the existing sync cursor with the watch response historyId.
* User confirmed push-triggered sync should run even if the normal sync worker is not currently active.
* User confirmed 30-second per-account coalescing for duplicate Pub/Sub-triggered sync work.
* User confirmed deployment documentation is part of MVP.
* User confirmed this task should keep the existing Gmail OAuth scope.
* Existing settings UI has a global realtime preference in `src/features/settings/GeneralTab.tsx`.
* Existing account list UI in `src/features/settings/AccountsTab.tsx` already renders per-account realtime status text.
* No Gmail-specific "enable realtime push" control exists yet.
* `accounts.sync_state` is already a JSON column and `pebble_store::SyncState` preserves unknown `extra` fields during read-modify-write updates.
* Current migrations already include `auth_data` and `sync_state`; adding provider-specific Gmail watch metadata can avoid a schema migration if it lives under `sync_state.extra`.

## Research References

* [`research/google-gmail-pubsub.md`](research/google-gmail-pubsub.md) — Official Gmail/Pub/Sub docs confirm watch response fields, label filtering, and authenticated push behavior.
