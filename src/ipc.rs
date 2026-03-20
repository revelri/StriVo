use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::DaemonEvent;
use crate::platform::{ChannelEntry, PlatformKind};
use crate::recording::job::RecordingJob;
use crate::recording::RecordingCommand;

/// Messages sent from TUI client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Request full state snapshot
    Hello,
    /// Forward a recording command
    Recording(RecordingCommand),
    /// Trigger immediate channel poll
    PollNow,
    /// Graceful daemon shutdown
    Shutdown,
}

/// Messages sent from daemon to TUI client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Full state snapshot (sent in response to Hello)
    StateSnapshot {
        channels: Vec<ChannelEntry>,
        recordings: HashMap<Uuid, RecordingJob>,
        twitch_connected: bool,
        youtube_connected: bool,
        patreon_connected: bool,
        pending_auth: Option<(PlatformKind, String, String)>,
    },
    /// Incremental update event
    Event(DaemonEvent),
}

/// Socket path for the daemon
pub fn socket_path() -> std::path::PathBuf {
    crate::config::AppConfig::state_dir().join("strivo.sock")
}

/// PID file path for the daemon
pub fn pid_path() -> std::path::PathBuf {
    crate::config::AppConfig::state_dir().join("strivo.pid")
}

/// Write a message as newline-delimited JSON
pub fn encode_message<T: Serialize>(msg: &T) -> Result<String, serde_json::Error> {
    let mut s = serde_json::to_string(msg)?;
    s.push('\n');
    Ok(s)
}

/// Check if the daemon is running by checking the PID file
pub fn is_daemon_running() -> bool {
    let pid_file = pid_path();
    if !pid_file.exists() {
        return false;
    }
    match std::fs::read_to_string(&pid_file) {
        Ok(pid_str) => {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // Check if process exists
                unsafe { libc::kill(pid, 0) == 0 }
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
