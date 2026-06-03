use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

const SESSION_TTL: Duration = Duration::from_secs(7 * 24 * 3600);
const RATE_LIMIT_MAX_FAILURES: u32 = 5;
const RATE_LIMIT_LOCK_DURATION: Duration = Duration::from_secs(15 * 60);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub created_at: Instant,
}

struct RateLimitEntry {
    failures: u32,
    lock_until: Option<Instant>,
    updated_at: Instant,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SessionCleanupStats {
    pub sessions_removed: usize,
    pub rate_limits_removed: usize,
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
        let Some(entry) = limits.get_mut(source) else {
            return true;
        };

        if let Some(lock_until) = entry.lock_until {
            if now >= lock_until {
                limits.remove(source);
                return true;
            }
            return false;
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
            updated_at: now,
        });

        entry.updated_at = now;
        entry.failures += 1;
        if entry.failures >= RATE_LIMIT_MAX_FAILURES {
            entry.lock_until = Some(now + RATE_LIMIT_LOCK_DURATION);
            false
        } else {
            true
        }
    }

    pub async fn cleanup_expired(&self) -> SessionCleanupStats {
        let mut sessions = self.sessions.lock().await;
        let before_sessions = sessions.len();
        sessions.retain(|_, info| info.created_at.elapsed() < SESSION_TTL);
        let sessions_removed = before_sessions - sessions.len();
        drop(sessions);

        let mut rate_limits = self.rate_limits.lock().await;
        let now = Instant::now();
        let before_rate_limits = rate_limits.len();
        rate_limits.retain(|_, entry| {
            if let Some(lock_until) = entry.lock_until {
                return now < lock_until;
            }
            now.duration_since(entry.updated_at) < RATE_LIMIT_LOCK_DURATION
        });
        let rate_limits_removed = before_rate_limits - rate_limits.len();

        SessionCleanupStats {
            sessions_removed,
            rate_limits_removed,
        }
    }
}

pub fn spawn_session_cleanup_task(store: Arc<SessionStore>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
        loop {
            interval.tick().await;
            let stats = store.cleanup_expired().await;
            if stats.sessions_removed > 0 || stats.rate_limits_removed > 0 {
                tracing::info!(
                    sessions_removed = stats.sessions_removed,
                    rate_limits_removed = stats.rate_limits_removed,
                    "Expired auth state cleaned"
                );
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn check_rate_limit_does_not_create_empty_entries() {
        let store = SessionStore::new("hash".to_string());

        assert!(store.check_rate_limit("127.0.0.1").await);
        assert!(store.rate_limits.lock().await.is_empty());
    }

    #[tokio::test]
    async fn cleanup_expired_removes_old_sessions_and_rate_limits() {
        let store = SessionStore::new("hash".to_string());
        store.sessions.lock().await.insert(
            "old".to_string(),
            SessionInfo {
                created_at: Instant::now() - SESSION_TTL - Duration::from_secs(1),
            },
        );
        store.sessions.lock().await.insert(
            "new".to_string(),
            SessionInfo {
                created_at: Instant::now(),
            },
        );
        store.rate_limits.lock().await.insert(
            "expired-lock".to_string(),
            RateLimitEntry {
                failures: RATE_LIMIT_MAX_FAILURES,
                lock_until: Some(Instant::now() - Duration::from_secs(1)),
                updated_at: Instant::now() - RATE_LIMIT_LOCK_DURATION,
            },
        );
        store.rate_limits.lock().await.insert(
            "fresh-failure".to_string(),
            RateLimitEntry {
                failures: 1,
                lock_until: None,
                updated_at: Instant::now(),
            },
        );

        let stats = store.cleanup_expired().await;

        assert_eq!(
            stats,
            SessionCleanupStats {
                sessions_removed: 1,
                rate_limits_removed: 1
            }
        );
        assert!(store.sessions.lock().await.contains_key("new"));
        assert!(!store.sessions.lock().await.contains_key("old"));
        assert!(store.rate_limits.lock().await.contains_key("fresh-failure"));
        assert!(!store.rate_limits.lock().await.contains_key("expired-lock"));
    }
}
