pub mod credentials;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_recording_dir")]
    pub recording_dir: PathBuf,

    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,

    pub twitch: Option<TwitchConfig>,
    pub youtube: Option<YouTubeConfig>,
    pub patreon: Option<PatreonConfig>,

    #[serde(default)]
    pub recording: RecordingConfig,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub auto_record_channels: Vec<AutoRecordEntry>,

    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,

    /// Tracks the path this config was loaded from, so save() can use it
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeConfig {
    pub client_id: String,
    pub client_secret: String,
    pub cookies_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatreonConfig {
    pub client_id: String,
    pub client_secret: String,
    #[serde(default = "default_patreon_poll_interval")]
    pub poll_interval_secs: u64,
}

fn default_patreon_poll_interval() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    #[serde(default)]
    pub transcode: bool,

    #[serde(default = "default_filename_template")]
    pub filename_template: String,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            transcode: false,
            filename_template: default_filename_template(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRecordEntry {
    pub platform: String,
    pub channel_id: String,
    pub channel_name: String,
}

/// Schedule-based recording entry.
/// Uses cron syntax for time-based recordings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub channel: String,
    pub cron: String,
    #[serde(default = "default_schedule_duration")]
    pub duration: String,
}

fn default_schedule_duration() -> String {
    "4h".to_string()
}

fn default_theme() -> String {
    "neon".to_string()
}

fn default_recording_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().join("Videos").join("StriVo"))
        .unwrap_or_else(|| PathBuf::from("./recordings"))
}

fn default_poll_interval() -> u64 {
    60
}

fn default_filename_template() -> String {
    "{channel}_{date}_{title}.mkv".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            recording_dir: default_recording_dir(),
            poll_interval_secs: default_poll_interval(),
            twitch: None,
            youtube: None,
            patreon: None,
            recording: RecordingConfig::default(),
            theme: default_theme(),
            auto_record_channels: Vec::new(),
            schedule: Vec::new(),
            config_path: None,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("", "", "strivo")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn cache_dir() -> PathBuf {
        directories::ProjectDirs::from("", "", "strivo")
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".cache"))
    }

    pub fn state_dir() -> PathBuf {
        directories::ProjectDirs::from("", "", "strivo")
            .map(|d| {
                d.state_dir()
                    .unwrap_or(d.data_dir())
                    .to_path_buf()
            })
            .unwrap_or_else(|| PathBuf::from(".state"))
    }

    pub fn load(path: Option<&std::path::Path>) -> Result<Self> {
        let path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(Self::config_path);

        if !path.exists() {
            let mut config = Self::default();
            config.config_path = Some(path.clone());
            config.save(Some(&path))?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let mut config: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;
        config.config_path = Some(path);
        Ok(config)
    }

    pub fn save(&self, path: Option<&std::path::Path>) -> Result<()> {
        let path = path
            .map(|p| p.to_path_buf())
            .or_else(|| self.config_path.clone())
            .unwrap_or_else(Self::config_path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }
}
