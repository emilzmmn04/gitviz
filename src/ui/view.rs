use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;
use super::widgets;

/// Height of the details panel (lines including border).
const DETAILS_HEIGHT: u16 = 9;
/// Height of the filter bar.
const FILTER_HEIGHT: u16 = 1;
/// Height of the help bar.
const HELP_HEIGHT: u16 = 1;

/// Top-level render function called each frame.
pub fn render(frame: &mut Frame, app: &App) {
    use crate::app::Mode;

    let area = frame.area();

    // Decide layout heights
    let filter_h = if app.mode == Mode::Filter { FILTER_HEIGHT } else { 0 };
    let details_h = if app.details_expanded { DETAILS_HEIGHT } else { 3 };

    // Build vertical layout
    let constraints = vec![
        Constraint::Min(3),              // graph list (greedy)
        Constraint::Length(details_h),   // details panel
        Constraint::Length(filter_h),    // filter bar (may be 0)
        Constraint::Length(HELP_HEIGHT), // help line
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

    widgets::render_help(frame, chunks[3]);
}
