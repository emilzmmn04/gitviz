use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;

use super::widgets;

const DETAILS_HEIGHT: u16 = 16;
const FILTER_HEIGHT: u16 = 1;
const HELP_HEIGHT: u16 = 1;

pub fn render(frame: &mut Frame, app: &mut App) {
    use crate::app::Mode;

    let area = frame.area();
    let filter_h = if app.mode == Mode::Filter { FILTER_HEIGHT } else { 0 };
    let details_h = if app.details_expanded { DETAILS_HEIGHT } else { 3 };

    let constraints = vec![
        Constraint::Min(3),
        Constraint::Length(details_h),
        Constraint::Length(filter_h),
        Constraint::Length(HELP_HEIGHT),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    widgets::render_graph(frame, app, chunks[0]);
    widgets::render_details(frame, app, chunks[1]);

    if app.mode == Mode::Filter {
        widgets::render_filter_bar(frame, app, chunks[2]);
    }

    widgets::render_help(frame, app, chunks[3]);

    if app.help_open {
        widgets::render_help_overlay(frame, app, centered_rect(72, 80, area));
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
