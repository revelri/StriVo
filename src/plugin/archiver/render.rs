use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::AppState;
use crate::tui::theme::Theme;

use super::ArchiverPlugin;
use super::types::{ArchiveState, ArchiverView, ConfigModalState, RecordingFilter};

pub fn render(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect, _app: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border_focused())
        .title(" Archiver ")
        .title_style(Theme::title());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [header_area, content_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    render_header(plugin, frame, header_area);

    match plugin.view {
        ArchiverView::ChannelList => render_channel_list(plugin, frame, content_area),
        ArchiverView::ArchiveQueue => render_queue(plugin, frame, content_area),
        ArchiverView::RecordingPicker => {} // Handled by render_pane() dispatch in mod.rs
    }

    render_footer(plugin, frame, footer_area);
}

fn render_header(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect) {
    let style_for = |v: ArchiverView| -> Style {
        if plugin.view == v {
            Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Theme::muted())
        }
    };

    let line = Line::from(vec![
        Span::styled(" Channels", style_for(ArchiverView::ChannelList)),
        Span::styled("  |  ", Style::new().fg(Theme::dim())),
        Span::styled("Queue", style_for(ArchiverView::ArchiveQueue)),
        Span::styled("  |  ", Style::new().fg(Theme::dim())),
        Span::styled("Picker", style_for(ArchiverView::RecordingPicker)),
        Span::styled("  [Tab]  [c] Settings", Style::new().fg(Theme::dim())),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_channel_list(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect) {
    if plugin.channels.is_empty() {
        let lines = vec![
            Line::raw(""),
            Line::styled(
                "  No channels available",
                Style::new().fg(Theme::secondary()).add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::styled(
                "  Connect Twitch or YouTube in StriVo to see channels here.",
                Style::new().fg(Theme::muted()),
            ),
        ];
        frame.render_widget(Paragraph::new(lines), area);
        return;
    }

    let items: Vec<ListItem> = plugin
        .channels
        .iter()
        .map(|ch| {
            let platform_label = match ch.platform.to_string().as_str() {
                "Twitch" => "TW",
                "YouTube" => "YT",
                _ => "??",
            };
            let name_display: String = ch.display_name.chars().take(30).collect();
            let pad = 32usize.saturating_sub(name_display.len() + platform_label.len() + 3);

            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(name_display, Style::new().fg(Theme::fg())),
                Span::raw(" ".repeat(pad)),
                Span::styled(
                    format!("({platform_label})"),
                    Style::new().fg(Theme::dim()),
                ),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(plugin.selected_channel));

    let list = List::new(items)
        .highlight_style(Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_queue(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect) {
    if plugin.jobs.is_empty() {
        let lines = vec![
            Line::raw(""),
            Line::styled(
                "  No archive jobs",
                Style::new().fg(Theme::muted()),
            ),
            Line::raw(""),
            Line::styled(
                "  Select a channel and press Enter to start archiving.",
                Style::new().fg(Theme::dim()),
            ),
        ];
        frame.render_widget(Paragraph::new(lines), area);
        return;
    }

    let items: Vec<ListItem> = plugin
        .jobs
        .iter()
        .map(|job| {
            let (indicator, ind_style) = match job.state {
                ArchiveState::Pending => ("○ ", Style::new().fg(Theme::dim())),
                ArchiveState::Scanning => ("⟳ ", Style::new().fg(Theme::secondary())),
                ArchiveState::Downloading => ("⟳ ", Style::new().fg(Theme::secondary())),
                ArchiveState::Paused => ("◼ ", Style::new().fg(Theme::secondary())),
                ArchiveState::Complete => ("✓ ", Style::new().fg(Theme::green())),
                ArchiveState::Failed => ("✗ ", Style::new().fg(Theme::red())),
            };

            let progress = if job.total_videos > 0 {
                let pct = (job.completed_videos as f64 / job.total_videos as f64 * 100.0) as u32;
                format!(" {}/{} ({}%)", job.completed_videos, job.total_videos, pct)
            } else {
                String::new()
            };

            let detail = match job.state {
                ArchiveState::Scanning => " Scanning...".to_string(),
                ArchiveState::Downloading => {
                    let current = job.current_video.as_deref().unwrap_or("");
                    let current_display: String = current.chars().take(25).collect();
                    format!("{progress} {current_display}")
                }
                ArchiveState::Complete => format!(" Complete ({} videos)", job.total_videos),
                ArchiveState::Failed => {
                    let err = job.error.as_deref().unwrap_or("unknown");
                    format!(" {}", err.chars().take(40).collect::<String>())
                }
                _ => String::new(),
            };

            let name_display: String = job.channel_name.chars().take(20).collect();

            ListItem::new(Line::from(vec![
                Span::styled("  ", Style::new()),
                Span::styled(indicator, ind_style),
                Span::styled(name_display, Style::new().fg(Theme::fg())),
                Span::styled(detail, Style::new().fg(Theme::muted())),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(plugin.selected_job));

    let list = List::new(items)
        .highlight_style(Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_footer(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect) {
    if let Some(ref error) = plugin.last_error {
        let error_display: String = error.chars().take(area.width.saturating_sub(4) as usize).collect();
        let line = Line::from(vec![
            Span::styled(" ⚠ ", Style::new().fg(Theme::red())),
            Span::styled(error_display, Style::new().fg(Theme::red())),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let active = plugin.jobs.iter().filter(|j| {
        j.state == ArchiveState::Downloading || j.state == ArchiveState::Scanning
    }).count();

    let archive_dir = plugin.config.as_ref()
        .map(|c| c.archive_dir.display().to_string())
        .unwrap_or_else(|| "~/Videos/StriVo/Archives".to_string());

    let dir_display: String = archive_dir.chars().take(area.width.saturating_sub(20) as usize).collect();

    let line = if active > 0 {
        Line::from(vec![
            Span::styled(format!(" [AR:{active}]"), Style::new().fg(Theme::secondary())),
            Span::styled(format!("  {dir_display}"), Style::new().fg(Theme::dim())),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {dir_display}"), Style::new().fg(Theme::dim())),
        ])
    };

    frame.render_widget(Paragraph::new(line), area);
}

// ──────────────────────────────────────────────
// New: Recording Picker, Config Modal
// ──────────────────────────────────────────────

/// Recording picker view for manual triggering / batch processing.
pub fn render_recording_picker(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect, app: &AppState) {
    let filter_label = match &plugin.picker.filter {
        RecordingFilter::All => "All".to_string(),
        RecordingFilter::ByChannel(ch) => format!("Channel: {ch}"),
        RecordingFilter::ByPlaylist(pl) => format!("Playlist: {pl}"),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border_focused())
        .title(format!(" Archiver Picker [{filter_label}] "))
        .title_style(Theme::title());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [content_area, footer_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).areas(inner);

    let finished: Vec<_> = plugin.picker.visible_ids.iter()
        .filter_map(|id| app.recordings.get(id))
        .collect();

    if finished.is_empty() {
        let lines = vec![
            Line::raw(""),
            Line::styled("  No finished recordings available", Style::new().fg(Theme::muted())),
            Line::raw(""),
            Line::styled("  Record a stream first, then select recordings to archive.", Style::new().fg(Theme::dim())),
        ];
        frame.render_widget(Paragraph::new(lines), content_area);
    } else {
        let mut lines = Vec::new();
        for (i, rec) in finished.iter().enumerate() {
            let is_selected = i == plugin.picker.selected;
            let is_checked = plugin.picker.selections.contains(&rec.id);
            let check = if is_checked { "[x]" } else { "[ ]" };
            let prefix = if is_selected { ">" } else { " " };

            let title = rec.stream_title.as_deref().unwrap_or("Untitled");
            let title_style = if is_selected {
                Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Theme::fg())
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{prefix} {check} "), Style::new().fg(if is_checked { Theme::green() } else { Theme::dim() })),
                Span::styled(&rec.channel_name, Style::new().fg(Theme::secondary())),
                Span::raw(" "),
                Span::styled(title, title_style),
            ]));
        }
        frame.render_widget(Paragraph::new(lines), content_area);
    }

    let sel_count = plugin.picker.selections.len();
    let hint = if sel_count > 0 {
        format!(" {sel_count} selected  [Enter] Archive  [Space] Toggle  [f] Filter  [a] Select all  [Esc] Back")
    } else {
        " [Enter] Archive  [Space] Select  [f] Filter  [a] Select all  [Tab] Views  [Esc] Back".to_string()
    };
    frame.render_widget(
        Paragraph::new(Line::styled(hint, Style::new().fg(Theme::muted()))),
        footer_area,
    );
}

/// Config modal overlay for the Archiver plugin.
pub fn render_config_modal(plugin: &ArchiverPlugin, frame: &mut Frame, area: Rect) {
    let ConfigModalState::Active { selected_field, editing, .. } = plugin.config_modal else {
        return;
    };

    let [_, center_v, _] = Layout::vertical([
        Constraint::Percentage(10),
        Constraint::Min(18),
        Constraint::Percentage(10),
    ]).areas(area);

    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage(15),
        Constraint::Min(50),
        Constraint::Percentage(15),
    ]).areas(center_v);

    frame.render_widget(Clear, center);

    let title = if plugin.configured {
        " Archiver Settings "
    } else {
        " Configure Archiver "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::border_focused())
        .title(title)
        .title_style(Theme::title());

    let inner = block.inner(center);
    frame.render_widget(block, center);

    let Some(ref draft) = plugin.config_draft else { return };

    let mut lines = Vec::new();
    let mut field_idx = 0usize;

    let add_field = |label: &str, value: &str, is_toggle: bool, idx: usize, sel: usize, edit: bool| -> Line<'static> {
        let is_sel = idx == sel;
        let prefix = if is_sel { " > " } else { "   " };
        let label_style = if is_sel {
            Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Theme::fg())
        };
        let val_style = if is_sel && edit && !is_toggle {
            Style::new().fg(Theme::secondary()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(Theme::dim())
        };

        Line::from(vec![
            Span::styled(prefix.to_string(), label_style),
            Span::styled(format!("{label}: "), label_style),
            Span::styled(value.to_string(), val_style),
            if is_sel && edit && !is_toggle { Span::styled("▌", Style::new().fg(Theme::primary())) } else { Span::raw("") },
        ])
    };

    // Field 0: Enabled
    lines.push(add_field("Enabled", if draft.enabled { "Yes" } else { "No" }, true, field_idx, selected_field, editing));
    field_idx += 1;

    // Field 1: Archive Dir
    lines.push(add_field("Archive Dir", &draft.archive_dir.display().to_string(), false, field_idx, selected_field, editing));
    field_idx += 1;

    // Field 2: Format
    lines.push(add_field("Format", &draft.format, false, field_idx, selected_field, editing));
    field_idx += 1;

    // Field 3: Concurrent Fragments
    lines.push(add_field("Fragments", &draft.concurrent_fragments.to_string(), false, field_idx, selected_field, editing));
    field_idx += 1;

    // Field 4: Rate Limit
    lines.push(add_field("Rate Limit", if draft.rate_limit.is_empty() { "(none)" } else { &draft.rate_limit }, false, field_idx, selected_field, editing));
    field_idx += 1;

    // Tandem channels header
    if !plugin.cached_channels.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled(
            "   Tandem Channels (auto-archive on recording finish):",
            Style::new().fg(Theme::secondary()),
        ));
    }

    // Channel checkboxes
    for (key, display) in &plugin.cached_channels {
        let is_sel = field_idx == selected_field;
        let is_checked = draft.tandem_channels.contains(key);
        let check = if is_checked { "[x]" } else { "[ ]" };
        let prefix = if is_sel { " > " } else { "   " };
        let style = if is_sel {
            Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD)
        } else if is_checked {
            Style::new().fg(Theme::green())
        } else {
            Style::new().fg(Theme::fg())
        };

        lines.push(Line::from(vec![
            Span::styled(prefix.to_string(), style),
            Span::styled(format!("{check} "), Style::new().fg(if is_checked { Theme::green() } else { Theme::dim() })),
            Span::styled(display.clone(), style),
        ]));
        field_idx += 1;
    }

    // Save button
    lines.push(Line::raw(""));
    let save_sel = field_idx == selected_field;
    let save_style = if save_sel {
        Style::new().fg(Theme::primary()).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Theme::secondary())
    };
    lines.push(Line::styled(
        if save_sel { "   [ Save ]" } else { "     Save" }.to_string(),
        save_style,
    ));

    let scroll_offset = if selected_field > inner.height as usize {
        selected_field.saturating_sub(inner.height as usize / 2)
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
