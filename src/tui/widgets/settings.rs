use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use crate::app::{ActivePane, AppState};
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState) {
    let border_style = app.pane_border(&ActivePane::Settings);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(" Settings ")
        .title_style(Theme::title());

    let settings_items = vec![
        (
            "Recording Directory",
            app.config.recording_dir.to_string_lossy().to_string(),
        ),
        (
            "Poll Interval",
            format!("{}s", app.config.poll_interval_secs),
        ),
        (
            "Transcode Mode",
            if app.transcode_mode {
                "ON (NVENC)".to_string()
            } else {
                "OFF (passthrough)".to_string()
            },
        ),
        (
            "Theme",
            Theme::current_name(),
        ),
        (
            "Twitch",
            if app.config.twitch.is_some() {
                if app.twitch_connected {
                    "Connected".to_string()
                } else {
                    "Configured (not connected)".to_string()
                }
            } else {
                "Not configured".to_string()
            },
        ),
        (
            "YouTube",
            if app.config.youtube.is_some() {
                if app.youtube_connected {
                    "Connected".to_string()
                } else {
                    "Configured (not connected)".to_string()
                }
            } else {
                "Not configured".to_string()
            },
        ),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .map(|(label, value)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {label}: "),
                    Style::new().fg(Theme::blue()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(value.as_str(), Style::new().fg(Theme::fg())),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.settings_selected));

    let config_path_hint = Line::from(vec![
        Span::raw(" Config: "),
        Span::styled(
            crate::config::AppConfig::config_path().to_string_lossy().to_string(),
            Style::new().fg(Theme::muted()),
        ),
    ]);

    let list = List::new(items)
        .block(block.title_bottom(config_path_hint))
        .highlight_style(Style::new().fg(Theme::bg()).bg(Theme::blue()));

    frame.render_stateful_widget(list, area, &mut state);
}
