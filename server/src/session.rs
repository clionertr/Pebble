// Session store and rate limiter for cookie-based authentication.
// Sessions live 7 days. Rate limiter: 5 failed attempts → 15 min lock per IP.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

const SESSION_TTL: Duration = Duration::from_secs(7 * 24 * 3600);
const RATE_LIMIT_MAX_FAILURES: u32 = 5;
const RATE_LIMIT_LOCK_DURATION: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub created_at: Instant,
}

struct RateLimitEntry {
    failures: u32,
    lock_until: Option<Instant>,
}

pub struct SessionStore {
    sessions: Mutex<HashMap<String, SessionInfo>>,
    rate_limits: Mutex<HashMap<String, RateLimitEntry>>,
    password_hash: String,
}

impl SessionStore {
    pub fn new(password_hash: String) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            rate_limits: Mutex::new(HashMap::new()),
            password_hash,
        }
    }

    pub fn check_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }

    pub async fn create_session(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        self.sessions.lock().await.insert(
            session_id.clone(),
            SessionInfo {
                created_at: Instant::now(),
            },
        );
        session_id
    }

    pub async fn validate_session(&self, session_id: &str) -> bool {
        let sessions = self.sessions.lock().await;
        match sessions.get(session_id) {
            Some(info) => {
                if info.created_at.elapsed() < SESSION_TTL {
                    true
                } else {
                    drop(sessions);
                    self.remove_session(session_id).await;
                    false
                }
            }
            None => false,
        }
    }

    pub async fn remove_session(&self, session_id: &str) {
        self.sessions.lock().await.remove(session_id);
    }

    /// Check rate limit for the given source. Returns true if allowed.
    pub async fn check_rate_limit(&self, source: &str) -> bool {
        let mut limits = self.rate_limits.lock().await;
        let now = Instant::now();
        let entry = limits.entry(source.to_string()).or_insert(RateLimitEntry {
            failures: 0,
            lock_until: None,
        });

        // Clean up expired locks
        if let Some(lock_until) = entry.lock_until {
            if now >= lock_until {
                entry.failures = 0;
                entry.lock_until = None;
            } else {
                return false;
            }
        }

        true
    }

    /// Record a failed login attempt. Returns true if further attempts are allowed.
    pub async fn record_failure(&self, source: &str) -> bool {
        let mut limits = self.rate_limits.lock().await;
        let now = Instant::now();
        let entry = limits.entry(source.to_string()).or_insert(RateLimitEntry {
            failures: 0,
            lock_until: None,
        });

        entry.failures += 1;
        if entry.failures >= RATE_LIMIT_MAX_FAILURES {
            entry.lock_until = Some(now + RATE_LIMIT_LOCK_DURATION);
            false
        } else {
            true
        }
    }
}
