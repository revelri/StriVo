use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::plugin::registry::PluginRegistry;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &AppState, registry: &PluginRegistry) {
    let Some(job_id) = app.show_properties else { return };
    let Some(rec) = app.recordings.get(&job_id) else { return };

    let [_, center_v, _] = Layout::vertical([
        Constraint::Percentage(10),
        Constraint::Min(20),
        Constraint::Percentage(10),
    ])
    .areas(area);

    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage(10),
        Constraint::Min(55),
        Constraint::Percentage(10),
    ])
    .areas(center_v);

    frame.render_widget(Clear, center);

    let title_text = rec.stream_title.as_deref().unwrap_or("Recording");
    let title_display: String = title_text.chars().take(40).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border_focused())
        .title(format!(" {title_display} "))
        .title_style(Theme::title());

    let inner = block.inner(center);
    frame.render_widget(block, center);

    let mut lines = Vec::new();

    // --- File Info ---
    lines.push(Line::styled(
        "  File Info",
        Style::new().fg(Theme::secondary()).add_modifier(Modifier::BOLD),
    ));

    let path_display: String = rec.output_path.display().to_string();
    let max_path = inner.width.saturating_sub(12) as usize;
    let path_truncated: String = if path_display.len() > max_path {
        format!("...{}", &path_display[path_display.len().saturating_sub(max_path)..])
    } else {
        path_display
    };
    lines.push(Line::from(vec![
        Span::styled("  Path:     ", Style::new().fg(Theme::dim())),
        Span::styled(path_truncated, Style::new().fg(Theme::fg())),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Size:     ", Style::new().fg(Theme::dim())),
        Span::styled(rec.format_size(), Style::new().fg(Theme::fg())),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Date:     ", Style::new().fg(Theme::dim())),
        Span::styled(rec.started_at.format("%Y-%m-%d %H:%M").to_string(), Style::new().fg(Theme::fg())),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Duration: ", Style::new().fg(Theme::dim())),
        Span::styled(rec.format_duration(), Style::new().fg(Theme::fg())),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Channel:  ", Style::new().fg(Theme::dim())),
        Span::styled(&rec.channel_name, Style::new().fg(Theme::fg())),
        Span::styled(format!(" ({})", rec.platform), Style::new().fg(Theme::dim())),
    ]));

    // --- Media Info (from ffprobe cache) ---
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Media Info",
        Style::new().fg(Theme::secondary()).add_modifier(Modifier::BOLD),
    ));

    if let Some(info) = app.media_info_cache.get(&job_id) {
        if let Some(ref codec) = info.video_codec {
            lines.push(Line::from(vec![
                Span::styled("  Video:    ", Style::new().fg(Theme::dim())),
                Span::styled(format!("{codec} {}", info.resolution_str()), Style::new().fg(Theme::fg())),
            ]));
        }
        if let Some(ref codec) = info.audio_codec {
            let sr = info.audio_sample_rate.map_or(String::new(), |r| format!(" {r}Hz"));
            lines.push(Line::from(vec![
                Span::styled("  Audio:    ", Style::new().fg(Theme::dim())),
                Span::styled(format!("{codec}{sr}"), Style::new().fg(Theme::fg())),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("  Bitrate:  ", Style::new().fg(Theme::dim())),
            Span::styled(info.bitrate_str(), Style::new().fg(Theme::fg())),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Format:   ", Style::new().fg(Theme::dim())),
            Span::styled(&info.format_name, Style::new().fg(Theme::fg())),
        ]));
    } else {
        lines.push(Line::styled(
            "  Probing...",
            Style::new().fg(Theme::muted()),
        ));
    }

    // --- Transcript Info (from Crunchr plugin) ---
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Transcript",
        Style::new().fg(Theme::secondary()).add_modifier(Modifier::BOLD),
    ));

    let crunchr_info = registry.plugin_ref("crunchr")
        .and_then(|p| p.as_any().downcast_ref::<crate::plugin::crunchr::CrunchrPlugin>())
        .and_then(|c| c.recording_info(&job_id.to_string()));

    if let Some(info) = crunchr_info {
        let status_color = if info.status == "complete" { Theme::green() } else { Theme::fg() };
        lines.push(Line::from(vec![
            Span::styled("  Status:   ", Style::new().fg(Theme::dim())),
            Span::styled(info.status.clone(), Style::new().fg(status_color)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Segments: ", Style::new().fg(Theme::dim())),
            Span::styled(info.segment_count.to_string(), Style::new().fg(Theme::fg())),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Words:    ", Style::new().fg(Theme::dim())),
            Span::styled(info.word_count.to_string(), Style::new().fg(Theme::fg())),
        ]));

        if info.has_analysis {
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                "  Analysis",
                Style::new().fg(Theme::secondary()).add_modifier(Modifier::BOLD),
            ));
            if let Some(ref summary) = info.summary {
                let max_summary = inner.width.saturating_sub(4) as usize;
                let summary_display: String = summary.chars().take(max_summary).collect();
                lines.push(Line::styled(
                    format!("  {summary_display}"),
                    Style::new().fg(Theme::fg()),
                ));
            }
            if !info.topics.is_empty() {
                let topics_str = info.topics.join(", ");
                lines.push(Line::from(vec![
                    Span::styled("  Topics:   ", Style::new().fg(Theme::dim())),
                    Span::styled(topics_str, Style::new().fg(Theme::primary())),
                ]));
            }
            if let Some(ref sentiment) = info.sentiment {
                let color = match sentiment.as_str() {
                    "positive" => Theme::green(),
                    "negative" => Theme::red(),
                    _ => Theme::muted(),
                };
                lines.push(Line::from(vec![
                    Span::styled("  Sentiment:", Style::new().fg(Theme::dim())),
                    Span::styled(format!(" {sentiment}"), Style::new().fg(color)),
                ]));
            }
        }
    } else {
        lines.push(Line::styled(
            "  Not processed",
            Style::new().fg(Theme::muted()),
        ));
    }

    // Footer
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("[Esc]", Theme::key_hint()),
        Span::raw(" Close  "),
        Span::styled("[i]", Theme::key_hint()),
        Span::raw(" Close"),
    ]));

    let scroll_offset = if lines.len() > inner.height as usize {
        lines.len().saturating_sub(inner.height as usize)
    } else {
        0
    };

    frame.render_widget(
        Paragraph::new(lines)
            .scroll((scroll_offset as u16, 0))
            .wrap(Wrap { trim: false }),
        inner,
    );
}
