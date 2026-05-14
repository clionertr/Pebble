# Google Gmail Pub/Sub Notes

Sources checked on 2026-05-14:

* Gmail `users.watch` reference: https://developers.google.com/workspace/gmail/api/reference/rest/v1/users/watch
* Gmail push notification guide: https://developers.google.com/workspace/gmail/api/guides/push
* Pub/Sub authenticated push subscriptions: https://docs.cloud.google.com/pubsub/docs/authenticate-push-subscriptions

## Findings

* `users.watch` is the Gmail API endpoint for setting or updating push notification watches.
* The watch request uses a fully qualified Pub/Sub topic name such as `projects/<project-id>/topics/<topic>`.
* The response includes the current mailbox `historyId` and an `expiration` timestamp in epoch milliseconds; renewal must happen before this time.
* `labelFilterBehavior` replaces the deprecated `labelFilterAction` field for label filtering.
* Pub/Sub authenticated push can include a Google-signed OIDC JWT in the `Authorization` header; receivers should verify signature, audience, expected service-account email, and `email_verified`.
* Google examples still pair Pub/Sub JWT validation with an endpoint verification token, so a shared secret can remain useful as a defense in depth, but it should not be the only trust signal for production if OIDC auth is available.

## Repo Mapping

* Pebble already has a Gmail history cursor in `accounts.sync_state.last_sync_cursor`.
* `crates/pebble-mail/src/gmail_sync.rs` already implements `poll_changes()` using Gmail history, message fetch, local storage, attachment persistence, and message event emission.
* `crates/pebble-mail/src/realtime_policy.rs` already defines `SyncTrigger::ProviderPush`, and `src-tauri/src/rpc/sync_cmd.rs` can route string reason `"provider_push"` to the running worker.
* The missing pieces are watch registration/renewal, Pub/Sub webhook verification/decoding, mapping `emailAddress` to account id, and triggering or starting the existing Gmail sync worker.
