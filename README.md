# StriVo

TUI-native live stream PVR. Monitor channels across Twitch, YouTube, and Patreon — automatically record when they go live, play back via mpv, and optionally transcribe recordings with Whisper.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange)]()
[![Version](https://img.shields.io/badge/version-0.3.0-blue)]()

---

## What it does

StriVo runs in your terminal (or as a background daemon) and watches your followed channels. When a stream goes live, it records via FFmpeg and notifies you. You can browse recordings, play them back through mpv, and search across your archive — all without leaving the terminal.

**Platform support:**

| Platform | Auth | Monitoring | Recording | Notes |
|----------|------|-----------|-----------|-------|
| Twitch | OAuth device flow | Channel follows polling | FFmpeg + streamlink | Sub-only streams via OAuth token passthrough |
| YouTube | OAuth + Data API v3 | Broadcast detection | FFmpeg + yt-dlp | Cookie-based auth for members-only |
| Patreon | Membership API | Post/stream detection | yt-dlp | Subscription tier extraction |

## Features

**Core:**
- Multi-platform channel monitoring with configurable poll intervals
- Automatic recording when channels go live (per-channel toggle)
- Live playback through mpv without downloading first
- Cron-based recording schedules
- Desktop notifications on go-live events
- Configurable filename templates (`{channel}_{date}_{title}.mkv`)
- Intelligent retry with backoff on failed recordings

**TUI:**
- Sidebar with channel list, auto-record toggles, platform indicators
- Channel detail view with stream metadata and recent recordings
- Recording browser — sortable, filterable, with size and duration
- Settings panel (edit config without leaving the TUI)
- Live log viewer
- First-run setup wizard for platform credentials
- Multiple color themes

**Daemon mode:**
- Background service via Unix socket IPC
- TUI clients connect to running daemon
- Systemd service generation (`strivo daemon install`)

**Plugins:**
- **Crunchr** — AI transcription and analysis
  - Backends: Whisper CLI, Voxtral (vLLM/RunPod), Mistral API, OpenRouter
  - Tandem mode: auto-trigger transcription on recording completion
  - Transcripts + analysis stored in SQLite
- **Archiver** — organize recordings by channel, render gallery views

## Requirements

- **Rust** 1.75+ to build
- **FFmpeg** — recording
- **mpv** — playback
- **streamlink** — Twitch stream resolution
- **yt-dlp** — YouTube/Patreon stream resolution

## Installation

```bash
git clone https://github.com/revelri/StriVo.git
cd StriVo
cargo build --release
```

Binary at `target/release/strivo`. Copy it to your PATH.

### Platform credentials

Run the setup wizard on first launch, or configure manually:

| Platform | How to get credentials |
|----------|----------------------|
| Twitch | Create an app at [dev.twitch.tv/console](https://dev.twitch.tv/console) — need `client_id` and `client_secret` |
| YouTube | Create OAuth credentials at [Google Cloud Console](https://console.cloud.google.com/) — need `client_id` and `client_secret` |
| Patreon | Uses membership API via browser cookies |

Credentials are stored in your OS keyring (macOS Keychain, GNOME Keyring, Windows Credential Manager).

## Usage

### TUI

```bash
strivo
```

Arrow keys + Enter to navigate. The sidebar shows monitored channels with live status indicators. Press `a` to toggle auto-record on a channel. Press `/` to search.

### Daemon

```bash
strivo daemon start     # start background service
strivo daemon stop      # stop
strivo daemon status    # check if running
strivo daemon install   # generate systemd service file
```

When the daemon is running, `strivo` launches as a client connecting via Unix socket.

### CLI

```bash
strivo config list              # show all settings
strivo config get <key>         # read a value
strivo config set <key> <val>   # write a value
strivo config path              # print config file location
strivo config reset             # restore defaults (keeps credentials)

strivo log tail [-l 100]        # live-tail the log
strivo log path                 # print log file location
strivo log clear                # wipe the log
```

### Flags

| Flag | Description |
|------|-------------|
| `-c, --config <path>` | Custom config file |
| `-l, --log-level <level>` | `trace`, `debug`, `info`, `warn`, `error` |

## Configuration

Config lives at `~/.config/strivo/config.toml` (XDG-compliant).

```toml
recording_dir = "/home/you/Videos/StriVo"
poll_interval_secs = 60

[twitch]
client_id = "..."
client_secret = "..."

[youtube]
client_id = "..."
client_secret = "..."
cookies_path = "/path/to/cookies.txt"   # optional, for members-only

[recording]
transcode = false
filename_template = "{channel}_{date}_{title}.mkv"

[[auto_record_channels]]
platform = "twitch"
channel_id = "12345"
channel_name = "streamer_name"

[[schedules]]
platform = "twitch"
channel_id = "12345"
cron = "0 20 * * 1-5"   # weekdays at 8pm
```

## Architecture

```
src/
  platform/          Trait-based abstraction (Twitch, YouTube, Patreon)
  monitor/           Channel polling, go-live detection
  recording/         Job lifecycle, FFmpeg/yt-dlp process management
  stream/            URL resolution via streamlink/yt-dlp
  playback/          mpv controller
  plugin/            Plugin trait, registry, lifecycle
    crunchr/         Transcription pipeline (Whisper/Mistral/Voxtral)
    archiver/        Recording organization and gallery
  tui/               ratatui rendering, event routing, themes
    widgets/         Sidebar, channel detail, recordings, settings, wizard
  daemon.rs          Background service, Unix socket listener
  ipc.rs             Client-server protocol
  config/            TOML config, OS keyring integration
```

**Design:**
- `Platform` trait abstracts service-specific APIs — adding a new platform means implementing one trait
- `RecordingManager` handles job lifecycle with retry and concurrent recording limits
- Plugin system runs async tasks triggered by recording events
- TUI renders via ratatui with a theme system for color customization
- Daemon/client split uses Unix socket IPC for zero-overhead communication

## License

[MIT](LICENSE)
