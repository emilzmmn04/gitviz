use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, DetailsTab};
use crate::git::model::{ChangeKind, Commit, CommitInspectData, Refs};
use crate::graph::{graph_prefix, GraphRow};
use crate::util::{format_iso, format_relative, short_hash};

pub fn render_graph(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&commit_idx| {
            let commit = &app.commits[commit_idx];
            let row = &app.graph[commit_idx];
            graph_list_item(app, commit, row, &app.refs)
        })
        .collect();

    let title = if app.filtered.len() == app.commits.len() {
        format!(" Commits ({}) ", app.commits.len())
    } else {
        format!(" Commits ({}/{}) ", app.filtered.len(), app.commits.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(title_style(app)),
        )
        .highlight_style(list_highlight_style(app))
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !app.filtered.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn graph_list_item<'a>(
    app: &'a App,
    commit: &'a Commit,
    row: &'a GraphRow,
    refs: &'a Refs,
) -> ListItem<'a> {
    let prefix = graph_prefix(row);
    let hash = short_hash(&commit.oid);
    let labels = refs.labels_for(&commit.oid);

    let mut spans: Vec<Span> = Vec::new();
    let prefix_style = if app.colors_enabled {
        Style::default().fg(lane_to_color(row.commit_lane))
    } else {
        Style::default()
    };

    spans.push(Span::styled(prefix, prefix_style));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(hash.to_string(), accent_style(app)));
    spans.push(Span::raw(" "));

    for label in &labels {
        spans.push(Span::styled(format!("[{}]", label), ref_style(app)));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::raw(commit.subject.clone()));
    ListItem::new(Line::from(spans))
}

pub fn render_details(frame: &mut Frame, app: &mut App, area: Rect) {
    let title = format!(" Details: {} ", app.active_tab.title());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(title_style(app));

    let inner_height = area.height.saturating_sub(2);
    let (content, content_height) = details_lines(app);
    let max_scroll = content_height.saturating_sub(inner_height);
    app.clamp_details_scroll(max_scroll);

    let paragraph = Paragraph::new(content)
        .block(block)
        .scroll((app.details_scroll, 0));
    frame.render_widget(paragraph, area);
}

fn details_lines(app: &App) -> (Vec<Line<'static>>, u16) {
    let mut lines = vec![tab_line(app), Line::from("")];

    let Some(commit) = app.selected_commit() else {
        lines.push(Line::from("No commits to display."));
        let len = lines.len() as u16;
        return (lines, len);
    };

    match app.active_tab {
        DetailsTab::Summary => {
            lines.extend(summary_lines(app, commit));
        }
        DetailsTab::Files => {
            lines.extend(files_lines(app));
        }
        DetailsTab::Diff => {
            lines.extend(diff_lines(app));
        }
    }

    let len = lines.len() as u16;
    (lines, len)
}

fn tab_line(app: &App) -> Line<'static> {
    let tabs = [DetailsTab::Summary, DetailsTab::Files, DetailsTab::Diff];
    let mut spans = Vec::new();
    for (idx, tab) in tabs.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::raw("  "));
        }
        let label = format!("[{}]", tab.title());
        let style = if *tab == app.active_tab {
            strong_style(app)
        } else {
            Style::default()
        };
        spans.push(Span::styled(label, style));
    }
    Line::from(spans)
}

fn summary_lines(app: &App, commit: &Commit) -> Vec<Line<'static>> {
    let labels = app.refs.labels_for(&commit.oid);
    let refs_value = if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(", ")
    };
    let parents_value = if commit.parents.is_empty() {
        "root commit".to_string()
    } else {
        commit
            .parents
            .iter()
            .map(|parent| short_hash(parent).to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let mut lines = vec![
        labeled_line(app, "Commit", commit.oid.clone()),
        labeled_line(app, "Author", format!("{} <{}>", commit.author, commit.author_email)),
        labeled_line(
            app,
            "Date",
            format!(
                "{}  ({})",
                format_iso(commit.timestamp),
                format_relative(commit.timestamp)
            ),
        ),
        labeled_line(app, "Parents", parents_value),
        labeled_line(app, "Refs", refs_value),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("    {}", commit.subject),
            strong_style(app),
        )]),
        labeled_line(app, "Body", String::new()),
    ];

    if commit.body.trim().is_empty() {
        lines.push(Line::from("    (no body)"));
    } else {
        lines.extend(
            commit
                .body
                .lines()
                .take(8)
                .map(|line| Line::from(format!("    {}", line))),
        );
    }

    lines
}

fn files_lines(app: &App) -> Vec<Line<'static>> {
    if let Some(message) = app.selected_inspect_error() {
        return vec![Line::from(format!(
            "Failed to load commit details: {}",
            message
        ))];
    }

    let Some(data) = app.selected_inspect_data() else {
        return vec![Line::from("Loading changed files...")];
    };

    build_files_lines(data)
}

fn build_files_lines(data: &CommitInspectData) -> Vec<Line<'static>> {
    if data.changed_files.is_empty() {
        return vec![Line::from("(no changed files)")];
    }

    let mut lines: Vec<Line<'static>> = data
        .changed_files
        .iter()
        .map(|file| {
            let symbol = match file.change_kind {
                ChangeKind::Added => "A",
                ChangeKind::Modified => "M",
                ChangeKind::Deleted => "D",
                ChangeKind::Renamed => "R",
                ChangeKind::Copied => "C",
                ChangeKind::TypeChanged => "T",
                ChangeKind::Unmerged => "U",
                ChangeKind::Unknown(_) => "?",
            };

            let text = match &file.old_path {
                Some(old_path) => format!("{symbol}  {old_path} -> {}", file.path),
                None => format!("{symbol}  {}", file.path),
            };
            Line::from(text)
        })
        .collect();

    if data.file_list_truncated {
        lines.push(Line::from("... file list truncated"));
    }

    lines
}

fn diff_lines(app: &App) -> Vec<Line<'static>> {
    if let Some(message) = app.selected_inspect_error() {
        return vec![Line::from(format!(
            "Failed to load commit details: {}",
            message
        ))];
    }

    let Some(data) = app.selected_inspect_data() else {
        return vec![Line::from("Loading diff preview...")];
    };

    let mut lines: Vec<Line<'static>> = data
        .diff_text
        .lines()
        .map(|line| Line::from(line.to_string()))
        .collect();

    if data.diff_truncated && !data.diff_text.contains("... diff truncated;") {
        lines.push(Line::from(
            "... diff truncated; open in GitHub or use git show for the full patch",
        ));
    }

    lines
}

pub fn render_filter_bar(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.filter.is_empty() {
        "Search: (subject, body, author, hash, email, refs)".to_string()
    } else {
        format!("Search: {}_", app.filter)
    };

    let style = if app.colors_enabled {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
    };

    frame.render_widget(Paragraph::new(text).style(style), area);
}

pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let text = app.status_message.as_deref().unwrap_or(
        " j/k:move  Tab:tabs  y:copy  o:open  /:filter  r:reload  ?:help  q:quit ",
    );

    let style = if app.status_message.is_some() {
        accent_style(app)
    } else if app.colors_enabled {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    frame.render_widget(Paragraph::new(text).style(style), area);
}

pub fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .title_style(title_style(app));

    let lines = vec![
        Line::from(vec![Span::styled("Navigation", strong_style(app))]),
        Line::from("  j/k, arrows: move selection"),
        Line::from("  g / G: jump to top / bottom"),
        Line::from("  Enter: collapse or expand details"),
        Line::from(""),
        Line::from(vec![Span::styled("Search", strong_style(app))]),
        Line::from("  /: enter search"),
        Line::from("  Esc: clear search or close help"),
        Line::from("  n / N: next / previous search result"),
        Line::from(""),
        Line::from(vec![Span::styled("Tabs", strong_style(app))]),
        Line::from("  Tab / Shift-Tab: cycle detail tabs"),
        Line::from("  PageUp / PageDown: scroll details"),
        Line::from("  Ctrl-u / Ctrl-d: half-page scroll"),
        Line::from(""),
        Line::from(vec![Span::styled("Actions", strong_style(app))]),
        Line::from("  y: copy commit hash"),
        Line::from("  o: open commit in GitHub"),
        Line::from("  r: reload repository state"),
        Line::from(""),
        Line::from(vec![Span::styled("Quit", strong_style(app))]),
        Line::from("  ?: toggle help"),
        Line::from("  q: close help or quit"),
    ];

    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn labeled_line(app: &App, label: &str, value: String) -> Line<'static> {
    let mut spans = vec![Span::styled(format!("{label:<7} "), accent_style(app))];
    if !value.is_empty() {
        spans.push(Span::raw(value));
    }
    Line::from(spans)
}

fn lane_to_color(lane: usize) -> Color {
    const COLORS: [Color; 7] = [
        Color::Blue,
        Color::Red,
        Color::Green,
        Color::Magenta,
        Color::Cyan,
        Color::LightBlue,
        Color::LightRed,
    ];
    COLORS[lane % COLORS.len()]
}

fn title_style(app: &App) -> Style {
    if app.colors_enabled {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn accent_style(app: &App) -> Style {
    if app.colors_enabled {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn ref_style(app: &App) -> Style {
    if app.colors_enabled {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn strong_style(app: &App) -> Style {
    if app.colors_enabled {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn list_highlight_style(app: &App) -> Style {
    if app.colors_enabled {
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD)
    }
}
