use super::lanes::{GraphCell, GraphRow};

pub fn graph_prefix(row: &GraphRow) -> String {
    let mut output = String::with_capacity(row.cells.len() * 2);
    for (i, cell) in row.cells.iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }
        output.push(cell_glyph(*cell));
    }
    output
}

fn cell_glyph(cell: GraphCell) -> char {
    match cell {
        GraphCell::Empty => ' ',
        GraphCell::Vertical => '│',
        GraphCell::Horizontal => '─',
        GraphCell::Commit => '●',
        GraphCell::MergeCommit => '◎',
        GraphCell::CornerUpLeft => '╯',
        GraphCell::CornerUpRight => '╰',
        GraphCell::CornerDownLeft => '╮',
        GraphCell::CornerDownRight => '╭',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::lanes::GraphCell;

    #[test]
    fn test_graph_prefix_renders_merge_row() {
        let row = GraphRow {
            commit_lane: 0,
            cells: vec![GraphCell::MergeCommit, GraphCell::CornerDownLeft],
        };
        assert_eq!(graph_prefix(&row), "◎ ╮");
    }

    #[test]
    fn test_graph_prefix_renders_horizontal_connector() {
        let row = GraphRow {
            commit_lane: 2,
            cells: vec![
                GraphCell::CornerDownRight,
                GraphCell::Horizontal,
                GraphCell::Commit,
            ],
        };
        assert_eq!(graph_prefix(&row), "╭ ─ ●");
    }

}
