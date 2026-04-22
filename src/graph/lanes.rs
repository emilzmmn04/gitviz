use crate::git::model::Commit;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphCell {
    Empty,
    Vertical,
    Horizontal,
    Commit,
    MergeCommit,
    CornerUpLeft,
    CornerUpRight,
    CornerDownLeft,
    CornerDownRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRow {
    pub commit_lane: usize,
    pub cells: Vec<GraphCell>,
}

pub fn compute_layout(commits: &[Commit]) -> Vec<GraphRow> {
    let mut lanes: Vec<Option<String>> = Vec::new();
    let mut result = Vec::with_capacity(commits.len());

    for commit in commits {
        let commit_lane = lanes
            .iter()
            .position(|lane| lane.as_deref() == Some(commit.oid.as_str()))
            .unwrap_or_else(|| allocate_lane(&mut lanes));

        if commit_lane >= lanes.len() {
            lanes.resize(commit_lane + 1, None);
        }

        let active_before = snapshot_active(&lanes, commit_lane);

        lanes[commit_lane] = commit.parents.first().cloned();

        let mut extra_parent_lanes = Vec::new();
        for parent_oid in commit.parents.iter().skip(1) {
            let lane = lanes
                .iter()
                .position(|tracked| tracked.as_deref() == Some(parent_oid.as_str()))
                .unwrap_or_else(|| {
                    let lane = allocate_lane(&mut lanes);
                    lanes[lane] = Some(parent_oid.clone());
                    lane
                });
            extra_parent_lanes.push(lane);
        }

        let cols = lanes.len().max(commit_lane + 1);
        let mut cells = vec![GraphCell::Empty; cols];
        for (lane, is_active) in active_before.iter().copied().enumerate() {
            if is_active {
                cells[lane] = GraphCell::Vertical;
            }
        }

        cells[commit_lane] = if commit.parents.len() > 1 {
            GraphCell::MergeCommit
        } else {
            GraphCell::Commit
        };

        for parent_lane in extra_parent_lanes {
            if parent_lane > commit_lane {
                for cell in cells
                    .iter_mut()
                    .take(parent_lane)
                    .skip(commit_lane + 1)
                {
                    *cell = GraphCell::Horizontal;
                }
                cells[parent_lane] = GraphCell::CornerDownLeft;
            } else if parent_lane < commit_lane {
                for cell in cells
                    .iter_mut()
                    .take(commit_lane)
                    .skip(parent_lane + 1)
                {
                    *cell = GraphCell::Horizontal;
                }
                cells[parent_lane] = GraphCell::CornerDownRight;
            }
        }

        while matches!(cells.last(), Some(GraphCell::Empty)) {
            cells.pop();
        }
        while lanes.last() == Some(&None) {
            lanes.pop();
        }

        result.push(GraphRow { commit_lane, cells });
    }

    result
}

fn snapshot_active(lanes: &[Option<String>], commit_lane: usize) -> Vec<bool> {
    let mut active = Vec::with_capacity(lanes.len());
    for (i, lane) in lanes.iter().enumerate() {
        active.push(i == commit_lane || lane.is_some());
    }
    active
}

fn allocate_lane(lanes: &mut Vec<Option<String>>) -> usize {
    if let Some(pos) = lanes.iter().position(|lane| lane.is_none()) {
        pos
    } else {
        lanes.push(None);
        lanes.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(oid: &str, parents: &[&str]) -> Commit {
        Commit {
            oid: oid.to_string(),
            parents: parents.iter().map(|s| s.to_string()).collect(),
            author: "Test".to_string(),
            author_email: "t@t.com".to_string(),
            timestamp: 0,
            subject: "test".to_string(),
            body: String::new(),
        }
    }

    #[test]
    fn test_linear_history_stays_on_lane_zero() {
        let commits = vec![
            make_commit("c", &["b"]),
            make_commit("b", &["a"]),
            make_commit("a", &[]),
        ];
        let rows = compute_layout(&commits);
        assert!(rows.iter().all(|row| row.commit_lane == 0));
        assert_eq!(rows[0].cells, vec![GraphCell::Commit]);
    }

    #[test]
    fn test_merge_adds_right_connector() {
        let commits = vec![
            make_commit("M", &["A", "B"]),
            make_commit("A", &["root"]),
            make_commit("B", &["root"]),
            make_commit("root", &[]),
        ];
        let rows = compute_layout(&commits);
        assert_eq!(
            rows[0].cells,
            vec![
                GraphCell::MergeCommit,
                GraphCell::CornerDownLeft,
            ]
        );
    }
}
