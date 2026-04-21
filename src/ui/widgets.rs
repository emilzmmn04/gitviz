use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::App;
use crate::git::model::Refs;
use crate::graph::{graph_prefix, GraphNode};
use crate::util::{format_iso, format_relative, short_hash};

/// Render the scrollable commit graph list.
pub fn render_graph(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&commit_idx| {
            let commit = &app.commits[commit_idx];
            let node = &app.graph[commit_idx];
            graph_list_item(app, commit, node, &app.refs)
        })
        .collect();

    let title = if app.filtered.len() == app.commits.len() {
        format!(" Commits ({}) ", app.commits.len())
    } else {
        format!(
            " Commits ({}/{}) ",
            app.filtered.len(),
            app.commits.len()
        )
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
    commit: &'a crate::git::model::Commit,
    node: &'a GraphNode,
    refs: &'a Refs,
) -> ListItem<'a> {
    let prefix = graph_prefix(node);
    let hash = short_hash(&commit.oid);
    let labels = refs.labels_for(&commit.oid);

    let mut spans: Vec<Span> = Vec::new();

    // Graph prefix — colour by lane to give visual distinction
    let prefix_style = if app.colors_enabled {
        Style::default().fg(lane_to_color(node.lane))
    } else {
        Style::default()
    };
    spans.push(Span::styled(prefix, prefix_style));
    spans.push(Span::raw(" "));

    // Short hash
    spans.push(Span::styled(hash.to_string(), accent_style(app)));
    spans.push(Span::raw(" "));

    // Ref labels
    for label in &labels {
        spans.push(Span::styled(
            format!("[{}]", label),
            ref_style(app),
        ));
        spans.push(Span::raw(" "));
    }

    // Commit subject
    spans.push(Span::raw(commit.subject.clone()));

    ListItem::new(Line::from(spans))
}

/// Render the commit details panel at the bottom.
pub fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Details ")
        .title_style(title_style(app));

    let Some(commit) = app.selected_commit() else {
        let p = Paragraph::new("No commits to display.").block(block);
        frame.render_widget(p, area);
        return;
    };

    let hash_line = Line::from(vec![
        Span::styled("Commit  ", accent_style(app)),
        Span::styled(commit.oid.clone(), strong_style(app)),
    ]);

    let author_line = Line::from(vec![
        Span::styled("Author  ", accent_style(app)),
        Span::raw(format!("{} <{}>", commit.author, commit.author_email)),
    ]);

    let date_line = Line::from(vec![
        Span::styled("Date    ", accent_style(app)),
        Span::raw(format!(
            "{}  ({})",
            format_iso(commit.timestamp),
            format_relative(commit.timestamp)
        )),
    ]);

    let parents_line = if commit.parents.is_empty() {
        Line::from(vec![
            Span::styled("Parents ", accent_style(app)),
            Span::raw("root commit"),
        ])
    } else {
        let mut spans = vec![Span::styled("Parents ", accent_style(app))];
        for (i, parent) in commit.parents.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(", "));
            }
            spans.push(Span::raw(short_hash(parent).to_string()));
        }
        Line::from(spans)
    };

    let labels = app.refs.labels_for(&commit.oid);
    let refs_line = if labels.is_empty() {
        Line::from(vec![
            Span::styled("Refs    ", accent_style(app)),
            Span::raw("none"),
        ])
    } else {
        let mut spans = vec![Span::styled("Refs    ", accent_style(app))];
        for (i, l) in labels.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(", "));
            }
            spans.push(Span::styled(l.clone(), ref_style(app)));
        }
        Line::from(spans)
    };

    let blank = Line::from("");

    let subject_line = Line::from(vec![Span::styled(
        format!("    {}", commit.subject),
        strong_style(app),
    )]);

    let body_header = Line::from(vec![Span::styled("Body    ", accent_style(app))]);
    let body_preview = if commit.body.trim().is_empty() {
        vec![Line::from("    (no body)")]
    } else {
        commit
            .body
            .lines()
            .take(3)
            .map(|line| Line::from(format!("    {}", line)))
            .collect()
    };

    let mut text = vec![
        hash_line,
        author_line,
        date_line,
        parents_line,
        refs_line,
        blank,
        subject_line,
        body_header,
    ];
    text.extend(body_preview);
    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}

/// Render the filter input bar.
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
    let p = Paragraph::new(text).style(style);
    frame.render_widget(p, area);
}

/// Render a help line at the very bottom of the screen.
pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let text = " j/k:move  Enter:toggle details  /:filter  r:reload  Esc:clear  q:quit ";
    let style = if app.colors_enabled {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };
    let p = Paragraph::new(text).style(style);
    frame.render_widget(p, area);
}

/// Map a lane index to a terminal colour for visual variety.
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
