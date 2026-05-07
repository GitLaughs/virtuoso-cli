//! Heartbeat daemon — pings sessions to detect stale Virtuoso processes.
//!
//! Runs in the background and periodically checks that registered sessions
//! are still alive. Stale sessions are marked but not deleted (preserved for
//! user inspection/recovery).

use crate::client::bridge::VirtuosoClient;
use crate::error::Result;
use crate::models::SessionInfo;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Heartbeat state tracked per session.
#[derive(Debug)]
pub struct SessionHeartbeatState {
    pub session_id: String,
    pub last_heartbeat: SystemTime,
    pub is_stale: bool,
}

/// Heartbeat manager that periodically pings all registered sessions.
pub struct SessionHeartbeat {
    interval_secs: u64,
    stop_flag: Arc<AtomicBool>,
}

impl SessionHeartbeat {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            interval_secs,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the heartbeat loop in a background thread.
    /// Returns immediately; the thread runs until `stop()` is called or process exits.
    pub fn start(&self) {
        let interval_secs = self.interval_secs;
        let stop = self.stop_flag.clone();

        std::thread::spawn(move || {
            let interval = Duration::from_secs(interval_secs);
            tracing::info!("session heartbeat started (interval={}s)", interval_secs);

            loop {
                std::thread::sleep(interval);

                if stop.load(Ordering::SeqCst) {
                    tracing::info!("session heartbeat stopped");
                    break;
                }

                if let Err(e) = Self::check_all_sessions() {
                    tracing::warn!("heartbeat check failed: {e}");
                }
            }
        });
    }

    /// Signal the heartbeat thread to stop.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Check all local sessions, ping each, and update stale status.
    fn check_all_sessions() -> Result<()> {
        let sessions = SessionInfo::list().map_err(|e| {
            crate::error::VirtuosoError::Execution(format!("failed to list sessions: {e}"))
        })?;

        for session in sessions {
            let state = Self::ping_session(&session);
            if state.is_stale {
                tracing::warn!(
                    "session '{}' on port {} is stale (Virtuoso pid={} may have crashed)",
                    session.id,
                    session.port,
                    session.pid
                );
                // Mark session JSON as stale by updating the file
                if let Err(e) = Self::mark_stale(&session) {
                    tracing::warn!("failed to mark session '{}' as stale: {e}", session.id);
                }
            }
        }

        Ok(())
    }

    /// Ping a single session — returns heartbeat state.
    fn ping_session(session: &SessionInfo) -> SessionHeartbeatState {
        let client = VirtuosoClient::new(&session.host, session.port, 5000);
        let is_alive = client.ping().is_ok();

        SessionHeartbeatState {
            session_id: session.id.clone(),
            last_heartbeat: SystemTime::now(),
            is_stale: !is_alive,
        }
    }

    /// Mark a session as stale by appending `.stale` flag file.
    fn mark_stale(session: &SessionInfo) -> Result<()> {
        let dir = SessionInfo::sessions_dir();
        let stale_flag = dir.join(format!("{}.stale", session.id));
        std::fs::write(&stale_flag, "").map_err(|e| {
            crate::error::VirtuosoError::Execution(format!("failed to write stale flag: {e}"))
        })
    }

    /// Remove stale flag for a session (called on successful reconnect).
    pub fn clear_stale(session_id: &str) -> Result<()> {
        let dir = SessionInfo::sessions_dir();
        let stale_flag = dir.join(format!("{}.stale", session_id));
        if stale_flag.exists() {
            std::fs::remove_file(&stale_flag).map_err(|e| {
                crate::error::VirtuosoError::Execution(format!("failed to remove stale flag: {e}"))
            })?;
        }
        Ok(())
    }
}

/// Returns true if a session is marked stale.
pub fn is_session_stale(session_id: &str) -> bool {
    let dir = SessionInfo::sessions_dir();
    dir.join(format!("{}.stale", session_id)).exists()
}
