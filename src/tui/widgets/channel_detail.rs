use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::{ActivePane, AppState};
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let focused = app.active_pane == ActivePane::Detail;
    let border_style = if focused {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(" Channel Detail ")
        .title_style(Theme::title());

    let Some(channel) = app.selected_channel() else {
        let placeholder = Paragraph::new("Select a channel from the sidebar")
            .style(Style::new().fg(Theme::muted()))
            .block(block);
        frame.render_widget(placeholder, area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Responsive layout: horizontal if wide enough, else vertical
    let (info_area, thumbnail_area) = if inner.width >= 70 {
        let [info, thumb] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(46),
        ])
        .areas(inner);
        (info, thumb)
    } else {
        let [info, thumb] = Layout::vertical([
            Constraint::Length(7),
            Constraint::Fill(1),
        ])
        .areas(inner);
        (info, thumb)
    };

    // Stream info
    let title = channel
        .stream_title
        .as_deref()
        .unwrap_or("Not streaming");
    let category = channel
        .game_or_category
        .as_deref()
        .unwrap_or("");
    let viewers = channel
        .viewer_count
        .map(|v| format!("{} viewers", format_count(v)))
        .unwrap_or_default();
    let uptime = channel
        .started_at
        .map(|s| format_duration(chrono::Utc::now() - s))
        .unwrap_or_default();

    let status_indicator = if channel.is_live {
        Span::styled(
            " LIVE ",
            Style::new()
                .fg(Theme::bg())
                .bg(Theme::green())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" OFFLINE ", Style::new().fg(Theme::fg()).bg(Theme::dim()))
    };

    // Check if currently recording
    let is_recording = app.is_channel_recording(&channel.id);

    let rec_indicator = if is_recording {
        Span::styled(
            " REC ",
            Style::new()
                .fg(Theme::bg())
                .bg(Theme::red())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::raw("")
    };

    let auto_indicator = if channel.auto_record {
        Span::styled(" MON ", Style::new().fg(Theme::bg()).bg(Theme::secondary()))
    } else {
        Span::raw("")
    };

    let info_lines = vec![
        Line::from(vec![
            Span::styled(
                &channel.display_name,
                Style::new().fg(Theme::fg()).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            status_indicator,
            Span::raw(" "),
            rec_indicator,
            Span::raw(" "),
            auto_indicator,
        ]),
        Line::raw(""),
        Line::styled(title, Style::new().fg(Theme::fg())),
        Line::from(vec![
            Span::styled(category, Style::new().fg(Theme::blue())),
            Span::styled(
                if !viewers.is_empty() {
                    format!(" · {viewers}")
                } else {
                    String::new()
                },
                Style::new().fg(Theme::muted()),
            ),
            Span::styled(
                if !uptime.is_empty() {
                    format!(" · {uptime}")
                } else {
                    String::new()
                },
                Style::new().fg(Theme::muted()),
            ),
        ]),
        Line::raw(""),
        Line::styled(
            format!("Platform: {}", channel.platform),
            Style::new().fg(Theme::muted()),
        ),
    ];

    frame.render_widget(
        Paragraph::new(info_lines).wrap(Wrap { trim: false }),
        info_area,
    );

    // Render thumbnail with rounded border
    let channel_id = channel.id.clone();
    let thumb_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border())
        .title(" Preview ")
        .title_style(Style::new().fg(Theme::muted()));
    let thumb_inner = thumb_block.inner(thumbnail_area);
    frame.render_widget(thumb_block, thumbnail_area);

    if let Some(proto) = app.thumbnail_protocols.get_mut(&channel_id) {
        let image_widget = ratatui_image::StatefulImage::default();
        frame.render_stateful_widget(image_widget, thumb_inner, proto);
    }
}

fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_duration(dur: chrono::TimeDelta) -> String {
    let total_secs = dur.num_seconds();
    if total_secs < 0 {
        return String::new();
    }
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
