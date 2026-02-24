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
            graph_list_item(commit_idx, commit, node, &app.refs)
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
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !app.filtered.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(list, area, &mut state);
}

fn graph_list_item<'a>(
    _commit_idx: usize,
    commit: &'a crate::git::model::Commit,
    node: &'a GraphNode,
    refs: &'a Refs,
) -> ListItem<'a> {
    let prefix = graph_prefix(node);
    let hash = short_hash(&commit.oid);
    let labels = refs.labels_for(&commit.oid);

    let mut spans: Vec<Span> = Vec::new();

    // Graph prefix — colour by lane to give visual distinction
    let lane_color = lane_to_color(node.lane);
    spans.push(Span::styled(prefix, Style::default().fg(lane_color)));
    spans.push(Span::raw(" "));

    // Short hash
    spans.push(Span::styled(
        hash.to_string(),
        Style::default().fg(Color::Yellow),
    ));
    spans.push(Span::raw(" "));

    // Ref labels
    for label in &labels {
        spans.push(Span::styled(
            format!("[{}]", label),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
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
        .title_style(Style::default().fg(Color::Cyan));

    let Some(commit) = app.selected_commit() else {
        let p = Paragraph::new("No commits to display.").block(block);
        frame.render_widget(p, area);
        return;
    };

    let hash_line = Line::from(vec![
        Span::styled("Commit  ", Style::default().fg(Color::Yellow)),
        Span::styled(
            commit.oid.clone(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]);

    let author_line = Line::from(vec![
        Span::styled("Author  ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("{} <{}>", commit.author, commit.author_email)),
    ]);

    let date_line = Line::from(vec![
        Span::styled("Date    ", Style::default().fg(Color::Yellow)),
        Span::raw(format!(
            "{}  ({})",
            format_iso(commit.timestamp),
            format_relative(commit.timestamp)
        )),
    ]);

    let labels = app.refs.labels_for(&commit.oid);
    let refs_line = if labels.is_empty() {
        Line::from(vec![
            Span::styled("Refs    ", Style::default().fg(Color::Yellow)),
            Span::raw("—"),
        ])
    } else {
        let mut spans = vec![Span::styled("Refs    ", Style::default().fg(Color::Yellow))];
        for (i, l) in labels.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(", "));
            }
            spans.push(Span::styled(
                l.clone(),
                Style::default().fg(Color::Green),
            ));
        }
        Line::from(spans)
    };

    let blank = Line::from("");

    let subject_line = Line::from(vec![Span::styled(
        format!("    {}", commit.subject),
        Style::default().add_modifier(Modifier::BOLD),
    )]);

    let text = vec![hash_line, author_line, date_line, refs_line, blank, subject_line];
    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}

/// Render the filter input bar.
pub fn render_filter_bar(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.filter.is_empty() {
        "Search: (type to filter by commit message)".to_string()
    } else {
        format!("Search: {}_", app.filter)
    };

    let style = Style::default().fg(Color::Black).bg(Color::Yellow);
    let p = Paragraph::new(text).style(style);
    frame.render_widget(p, area);
}

/// Render a help line at the very bottom of the screen.
pub fn render_help(frame: &mut Frame, area: Rect) {
    let text = " j/k:move  Enter:toggle details  /:filter  Esc:clear  q:quit ";
    let p = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
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
