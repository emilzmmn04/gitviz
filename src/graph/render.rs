use super::lanes::GraphNode;

/// Build the graph prefix string for a commit row.
///
/// Each lane occupies one character column. Lanes are separated by a single space.
/// The commit's lane shows `●` (or `◎` for merge commits).
/// Other active lanes show `│`.
/// Inactive lanes show ` `.
///
/// Example (4 lanes, commit on lane 1, merge):
///   `│ ◎ │  `
pub fn graph_prefix(node: &GraphNode) -> String {
    let cols = node.total_lanes.max(node.lane + 1);
    let mut s = String::with_capacity(cols * 2);

    for i in 0..cols {
        if i > 0 {
            s.push(' ');
        }
        if i == node.lane {
            if node.is_merge {
                s.push('◎');
            } else {
                s.push('●');
            }
        } else if node.active.get(i).copied().unwrap_or(false) {
            s.push('│');
        } else {
            s.push(' ');
        }
    }

    s
}

/// Width (in terminal columns) of the graph prefix for `total_lanes` lanes.
#[allow(dead_code)]
pub fn prefix_width(total_lanes: usize) -> usize {
    if total_lanes == 0 {
        1
    } else {
        total_lanes * 2 - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::lanes::GraphNode;

    fn node(lane: usize, total: usize, active: Vec<bool>, merge: bool) -> GraphNode {
        GraphNode {
            lane,
            is_merge: merge,
            total_lanes: total,
            active,
        }
    }

    #[test]
    fn test_single_lane() {
        let n = node(0, 1, vec![true], false);
        assert_eq!(graph_prefix(&n), "●");
    }

    #[test]
    fn test_second_lane() {
        let n = node(1, 2, vec![true, true], false);
        assert_eq!(graph_prefix(&n), "│ ●");
    }

    #[test]
    fn test_merge_node() {
        let n = node(0, 2, vec![true, true], true);
        assert_eq!(graph_prefix(&n), "◎ │");
    }

    #[test]
    fn test_inactive_lanes() {
        // Lane 0 active, lane 1 inactive, lane 2 is commit
        let n = node(2, 3, vec![true, false, true], false);
        assert_eq!(graph_prefix(&n), "│   ●");
    }
}
