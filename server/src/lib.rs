// Library root for integration tests.
// The binary entry point is main.rs.
// Both lib.rs and main.rs declare the same modules — lib.rs exposes them
// as public for integration tests; main.rs uses them for the binary.

pub mod account_colors;
pub mod api;
pub mod auth;
pub mod events;
pub mod gmail_realtime;
pub mod mail_latency;
pub mod middleware;
pub mod realtime;
pub mod rpc;
pub mod session;
pub mod snooze_watcher;
pub mod state;
