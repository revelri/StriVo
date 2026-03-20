use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::sync::{OnceLock, RwLock};

/// Global theme storage — initialized once, switchable at runtime.
static THEME: OnceLock<RwLock<ThemeData>> = OnceLock::new();
/// Generation counter — incremented on every theme switch.
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// Per-frame cache — avoids 100+ RwLock reads + ThemeData clones per render.
thread_local! {
    static CACHED: RefCell<(u64, ThemeData)> = RefCell::new((u64::MAX, ThemeData::neon()));
}

/// Platform colors — fixed, NOT themeable per DESIGN.md.
pub const TWITCH_COLOR: Color = Color::Rgb(145, 70, 255);
pub const YOUTUBE_COLOR: Color = Color::Rgb(255, 0, 0);
pub const PATREON_COLOR: Color = Color::Rgb(255, 66, 77);

/// 16 semantic color slots per DESIGN.md's Ghostty-style theming spec.
///
/// Slots: bg, fg, surface, overlay, primary, secondary, dim, muted,
///        + 8 ANSI colors (black, red, green, yellow, blue, magenta, cyan, white).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeData {
    pub name: String,

    // Base surfaces
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub bg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub fg: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub surface: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub overlay: Color,

    // Semantic accents
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub primary: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub secondary: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub dim: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub muted: Color,

    // ANSI color slots
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_black: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_red: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_green: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_yellow: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_blue: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_magenta: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_cyan: Color,
    #[serde(deserialize_with = "de_color", serialize_with = "ser_color")]
    pub ansi_white: Color,
}

fn ser_color<S: serde::Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
    match color {
        Color::Rgb(r, g, b) => s.serialize_str(&format!("#{r:02x}{g:02x}{b:02x}")),
        _ => s.serialize_str("#000000"),
    }
}

fn de_color<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    let s = String::deserialize(d)?;
    parse_hex_color(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid hex color: {s}")))
}

pub fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

// ── Built-in themes (per DESIGN.md) ─────────────────────────────────────

impl ThemeData {
    /// Neon (default) — Cyan + Amber on deep blue-black. The signature StriVo look.
    pub fn neon() -> Self {
        Self {
            name: "neon".to_string(),
            bg: Color::Rgb(26, 27, 38),         // #1A1B26
            fg: Color::Rgb(232, 232, 226),       // #E8E8E2
            surface: Color::Rgb(36, 37, 58),     // #24253A
            overlay: Color::Rgb(59, 61, 86),     // #3B3D56
            primary: Color::Rgb(0, 229, 255),    // #00E5FF (cyan)
            secondary: Color::Rgb(255, 176, 32), // #FFB020 (amber)
            dim: Color::Rgb(86, 91, 126),        // #565B7E
            muted: Color::Rgb(169, 174, 207),    // #A9AECF
            ansi_black: Color::Rgb(26, 27, 38),
            ansi_red: Color::Rgb(255, 68, 68),   // #FF4444
            ansi_green: Color::Rgb(57, 255, 127), // #39FF7F
            ansi_yellow: Color::Rgb(255, 176, 32),
            ansi_blue: Color::Rgb(0, 180, 216),  // #00B4D8
            ansi_magenta: Color::Rgb(255, 121, 198), // #FF79C6
            ansi_cyan: Color::Rgb(0, 229, 255),
            ansi_white: Color::Rgb(232, 232, 226),
        }
    }

    /// Monochrome — Grayscale with red/green semantic colors only.
    pub fn monochrome() -> Self {
        Self {
            name: "monochrome".to_string(),
            bg: Color::Rgb(24, 24, 24),
            fg: Color::Rgb(220, 220, 220),
            surface: Color::Rgb(36, 36, 36),
            overlay: Color::Rgb(56, 56, 56),
            primary: Color::Rgb(200, 200, 200),   // white-ish accent
            secondary: Color::Rgb(160, 160, 160),
            dim: Color::Rgb(80, 80, 80),
            muted: Color::Rgb(140, 140, 140),
            ansi_black: Color::Rgb(24, 24, 24),
            ansi_red: Color::Rgb(220, 80, 80),
            ansi_green: Color::Rgb(80, 200, 80),
            ansi_yellow: Color::Rgb(200, 200, 80),
            ansi_blue: Color::Rgb(120, 120, 200),
            ansi_magenta: Color::Rgb(180, 120, 180),
            ansi_cyan: Color::Rgb(120, 200, 200),
            ansi_white: Color::Rgb(220, 220, 220),
        }
    }

    /// Catppuccin Mocha — Soothing pastels.
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "catppuccin-mocha".to_string(),
            bg: Color::Rgb(30, 30, 46),          // #1E1E2E
            fg: Color::Rgb(205, 214, 244),        // #CDD6F4
            surface: Color::Rgb(49, 50, 68),      // #313244
            overlay: Color::Rgb(69, 71, 90),      // #45475A
            primary: Color::Rgb(137, 180, 250),   // #89B4FA (blue)
            secondary: Color::Rgb(249, 226, 175), // #F9E2AF (yellow)
            dim: Color::Rgb(69, 71, 90),
            muted: Color::Rgb(108, 112, 134),     // #6C7086
            ansi_black: Color::Rgb(30, 30, 46),
            ansi_red: Color::Rgb(243, 139, 168),  // #F38BA8
            ansi_green: Color::Rgb(166, 227, 161), // #A6E3A1
            ansi_yellow: Color::Rgb(249, 226, 175),
            ansi_blue: Color::Rgb(137, 180, 250),
            ansi_magenta: Color::Rgb(245, 194, 231), // #F5C2E7
            ansi_cyan: Color::Rgb(148, 226, 213),  // #94E2D5
            ansi_white: Color::Rgb(205, 214, 244),
        }
    }

    /// Tokyo Night — Cool blues and muted tones.
    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo-night".to_string(),
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(192, 202, 245),        // #C0CAF5
            surface: Color::Rgb(36, 40, 59),      // #24283B
            overlay: Color::Rgb(52, 59, 88),      // #343B58
            primary: Color::Rgb(125, 207, 255),   // #7DCFFF (cyan)
            secondary: Color::Rgb(224, 175, 104), // #E0AF68 (amber)
            dim: Color::Rgb(52, 59, 88),
            muted: Color::Rgb(86, 95, 137),       // #565F89
            ansi_black: Color::Rgb(26, 27, 38),
            ansi_red: Color::Rgb(247, 118, 142),  // #F7768E
            ansi_green: Color::Rgb(158, 206, 106), // #9ECE6A
            ansi_yellow: Color::Rgb(224, 175, 104),
            ansi_blue: Color::Rgb(125, 207, 255),
            ansi_magenta: Color::Rgb(187, 154, 247), // #BB9AF7
            ansi_cyan: Color::Rgb(125, 207, 255),
            ansi_white: Color::Rgb(192, 202, 245),
        }
    }

    /// Solarized Dark — Ethan Schoonover's precision palette.
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            bg: Color::Rgb(0, 43, 54),            // #002B36
            fg: Color::Rgb(131, 148, 150),         // #839496
            surface: Color::Rgb(7, 54, 66),        // #073642
            overlay: Color::Rgb(88, 110, 117),     // #586E75
            primary: Color::Rgb(38, 139, 210),    // #268BD2 (blue)
            secondary: Color::Rgb(181, 137, 0),   // #B58900 (yellow)
            dim: Color::Rgb(88, 110, 117),
            muted: Color::Rgb(101, 123, 131),      // #657B83
            ansi_black: Color::Rgb(0, 43, 54),
            ansi_red: Color::Rgb(220, 50, 47),    // #DC322F
            ansi_green: Color::Rgb(133, 153, 0),  // #859900
            ansi_yellow: Color::Rgb(181, 137, 0),
            ansi_blue: Color::Rgb(38, 139, 210),
            ansi_magenta: Color::Rgb(211, 54, 130), // #D33682
            ansi_cyan: Color::Rgb(42, 161, 152),   // #2AA198
            ansi_white: Color::Rgb(238, 232, 213),  // #EEE8D5
        }
    }
}

/// Returns all built-in themes.
pub fn builtin_themes() -> Vec<ThemeData> {
    vec![
        ThemeData::neon(),
        ThemeData::monochrome(),
        ThemeData::catppuccin_mocha(),
        ThemeData::tokyo_night(),
        ThemeData::solarized_dark(),
    ]
}

/// Scan user theme directory for .toml theme files.
pub fn scan_user_themes() -> Vec<ThemeData> {
    let themes_dir = crate::config::AppConfig::config_dir().join("themes");
    if !themes_dir.exists() {
        return Vec::new();
    }

    let mut themes = Vec::new();
    let entries = match std::fs::read_dir(&themes_dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("Failed to read themes directory: {e}");
            return Vec::new();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<ThemeData>(&contents) {
                Ok(theme) => {
                    tracing::info!("Loaded user theme '{}' from {}", theme.name, path.display());
                    themes.push(theme);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse theme {}: {e}", path.display());
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read theme {}: {e}", path.display());
            }
        }
    }

    themes
}

/// List all available theme names (user themes override built-ins).
pub fn available_themes() -> Vec<String> {
    let user = scan_user_themes();
    let builtins = builtin_themes();

    let mut names: Vec<String> = user.iter().map(|t| t.name.clone()).collect();
    for b in &builtins {
        if !names.contains(&b.name) {
            names.push(b.name.clone());
        }
    }
    names
}

/// Resolve a theme by name: user files > built-ins > default.
pub fn resolve_theme(name: &str) -> ThemeData {
    for theme in scan_user_themes() {
        if theme.name == name {
            return theme;
        }
    }
    for theme in builtin_themes() {
        if theme.name == name {
            return theme;
        }
    }
    ThemeData::neon()
}

// ── Theme accessor (per-frame cached) ───────────────────────────────────

pub struct Theme;

#[allow(dead_code)]
impl Theme {
    /// Initialize the global theme. Call once at startup.
    pub fn init(theme_name: &str) {
        let data = resolve_theme(theme_name);
        let _ = THEME.set(RwLock::new(data));
    }

    /// Switch the global theme at runtime.
    pub fn set(theme_name: &str) {
        let data = resolve_theme(theme_name);
        if let Some(lock) = THEME.get() {
            if let Ok(mut guard) = lock.write() {
                *guard = data;
                GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Get the current theme name.
    pub fn current_name() -> String {
        Self::cached().name.clone()
    }

    /// Get a cached copy of the current theme data. Reads the RwLock at most
    /// once per generation (i.e., once per theme switch), not once per call.
    fn cached() -> ThemeData {
        let gen = GENERATION.load(std::sync::atomic::Ordering::Relaxed);
        CACHED.with(|cell| {
            let cached = cell.borrow();
            if cached.0 == gen {
                return cached.1.clone();
            }
            drop(cached);

            let data = THEME
                .get()
                .and_then(|lock| lock.read().ok())
                .map(|t| t.clone())
                .unwrap_or_else(ThemeData::neon);
            *cell.borrow_mut() = (gen, data.clone());
            data
        })
    }

    // ── Base surfaces ───────────────────────────────────────────────────
    pub fn bg() -> Color { Self::cached().bg }
    pub fn fg() -> Color { Self::cached().fg }
    pub fn surface() -> Color { Self::cached().surface }
    pub fn overlay() -> Color { Self::cached().overlay }

    // ── Semantic accents ────────────────────────────────────────────────
    pub fn primary() -> Color { Self::cached().primary }
    pub fn secondary() -> Color { Self::cached().secondary }
    pub fn dim() -> Color { Self::cached().dim }
    pub fn muted() -> Color { Self::cached().muted }

    // ── ANSI slots ──────────────────────────────────────────────────────
    pub fn red() -> Color { Self::cached().ansi_red }
    pub fn green() -> Color { Self::cached().ansi_green }
    pub fn yellow() -> Color { Self::cached().ansi_yellow }
    pub fn blue() -> Color { Self::cached().ansi_blue }
    pub fn magenta() -> Color { Self::cached().ansi_magenta }
    pub fn cyan() -> Color { Self::cached().ansi_cyan }

    // ── Platform colors (fixed constants, NOT themeable) ────────────────
    pub fn twitch() -> Color { TWITCH_COLOR }
    pub fn youtube() -> Color { YOUTUBE_COLOR }
    pub fn patreon() -> Color { PATREON_COLOR }

    // ── Derived: hotkey bar background ──────────────────────────────────
    fn hotkey_bg() -> Color {
        match Self::surface() {
            Color::Rgb(r, g, b) => Color::Rgb(
                r.saturating_add(10),
                g.saturating_add(10),
                b.saturating_add(14),
            ),
            other => other,
        }
    }

    // ── Style helpers ───────────────────────────────────────────────────
    pub fn title() -> Style {
        Style::new().fg(Self::primary()).add_modifier(Modifier::BOLD)
    }

    pub fn selected() -> Style {
        Style::new().fg(Self::bg()).bg(Self::primary())
    }

    pub fn status_live() -> Style {
        Style::new().fg(Self::green()).add_modifier(Modifier::BOLD)
    }

    pub fn status_recording() -> Style {
        Style::new().fg(Self::red()).add_modifier(Modifier::BOLD)
    }

    pub fn status_offline() -> Style {
        Style::new().fg(Self::muted())
    }

    pub fn border() -> Style {
        Style::new().fg(Self::dim())
    }

    pub fn border_focused() -> Style {
        Style::new().fg(Self::primary())
    }

    pub fn status_bar() -> Style {
        Style::new().fg(Self::fg()).bg(Self::overlay())
    }

    pub fn key_hint() -> Style {
        Style::new().fg(Self::secondary())
    }

    pub fn error() -> Style {
        Style::new().fg(Self::red())
    }

    pub fn hotkey_bar() -> Style {
        Style::new().fg(Self::fg()).bg(Self::hotkey_bg())
    }

    pub fn hotkey_key() -> Style {
        Style::new()
            .fg(Self::secondary())
            .bg(Self::hotkey_bg())
            .add_modifier(Modifier::BOLD)
    }

    pub fn day_header() -> Style {
        Style::new()
            .fg(Self::primary())
            .add_modifier(Modifier::BOLD)
    }

    pub fn stream_subtitle() -> Style {
        Style::new()
            .fg(Self::muted())
            .add_modifier(Modifier::ITALIC)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#ff5555"), Some(Color::Rgb(255, 85, 85)));
        assert_eq!(parse_hex_color("00e5ff"), Some(Color::Rgb(0, 229, 255)));
        assert_eq!(parse_hex_color("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(parse_hex_color("#FFFFFF"), Some(Color::Rgb(255, 255, 255)));
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("#fff"), None); // too short
        assert_eq!(parse_hex_color("#gggggg"), None); // invalid chars
        assert_eq!(parse_hex_color("#1234567"), None); // too long
    }

    #[test]
    fn test_all_builtin_themes_have_unique_names() {
        let themes = builtin_themes();
        let names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        let mut deduped = names.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(names.len(), deduped.len(), "duplicate theme names found");
    }

    #[test]
    fn test_resolve_theme_fallback() {
        let t = resolve_theme("nonexistent-theme");
        assert_eq!(t.name, "neon");
    }
}
