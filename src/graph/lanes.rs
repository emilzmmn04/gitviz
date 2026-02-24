use crate::git::model::Commit;

/// Per-commit layout information produced by the lane algorithm.
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Which lane (column) this commit sits on.
    pub lane: usize,
    /// True when this commit has more than one parent (merge commit).
    pub is_merge: bool,
    /// Total number of active lanes for this row (determines column count).
    pub total_lanes: usize,
    /// Which lanes are "active" (have a queued OID) when this row is rendered.
    /// Index corresponds to lane index.
    pub active: Vec<bool>,
}

/// Compute lane-based graph layout for a topo-ordered list of commits.
///
/// Algorithm (simplified git-log-graph style):
/// - Maintain `lanes: Vec<Option<String>>` where each slot holds the OID
///   we are "expecting" to appear in that lane next.
/// - For each commit:
///   1. Find the lane that holds `commit.oid`, or allocate a new one.
///   2. Record which lanes are currently active (for rendering vertical bars).
///   3. Update: set the commit's lane to its first parent.
///   4. For each additional parent (merge), find or allocate a lane.
///   5. Trim trailing empty lanes.
pub fn compute_layout(commits: &[Commit]) -> Vec<GraphNode> {
    // lanes[i] = Some(oid) means lane i is "expecting" commit with that oid
    let mut lanes: Vec<Option<String>> = Vec::new();
    let mut result = Vec::with_capacity(commits.len());

    for commit in commits {
        // --- Step 1: find or allocate this commit's lane ---
        let commit_lane = lanes
            .iter()
            .position(|l| l.as_deref() == Some(commit.oid.as_str()))
            .unwrap_or_else(|| allocate_lane(&mut lanes));

        // Ensure lanes vector is long enough
        if commit_lane >= lanes.len() {
            lanes.resize(commit_lane + 1, None);
        }

        // --- Step 2: snapshot active lanes BEFORE mutation ---
        let total = lanes.len();
        let active: Vec<bool> = (0..total)
            .map(|i| {
                if i == commit_lane {
                    true // commit node itself is "active" in this position
                } else {
                    lanes[i].is_some()
                }
            })
            .collect();

        let is_merge = commit.parents.len() > 1;

        // --- Step 3: update commit's lane to its first parent ---
        lanes[commit_lane] = commit.parents.first().cloned();

        // --- Step 4: handle additional parents (merge) ---
        for parent_oid in commit.parents.iter().skip(1) {
            // Only allocate a lane if this parent isn't already tracked
            if !lanes
                .iter()
                .any(|l| l.as_deref() == Some(parent_oid.as_str()))
            {
                let new_lane = allocate_lane(&mut lanes);
                lanes[new_lane] = Some(parent_oid.clone());
            }
        }

        // --- Step 5: trim trailing None lanes ---
        while lanes.last() == Some(&None) {
            lanes.pop();
        }

        result.push(GraphNode {
            lane: commit_lane,
            is_merge,
            total_lanes: total,
            active,
        });
    }

    result
}

/// Find the first unused (None) slot in `lanes`, or push a new slot.
fn allocate_lane(lanes: &mut Vec<Option<String>>) -> usize {
    if let Some(pos) = lanes.iter().position(|l| l.is_none()) {
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
        }
    }

    #[test]
    fn test_linear_history() {
        let commits = vec![
            make_commit("c", &["b"]),
            make_commit("b", &["a"]),
            make_commit("a", &[]),
        ];
        let nodes = compute_layout(&commits);
        // All commits should be on lane 0 for linear history
        assert!(nodes.iter().all(|n| n.lane == 0));
        assert!(nodes.iter().all(|n| !n.is_merge));
    }

    #[test]
    fn test_merge_commit() {
        // merge: M has parents A and B; B was on a side branch
        let commits = vec![
            make_commit("M", &["A", "B"]),
            make_commit("A", &["root"]),
            make_commit("B", &["root"]),
            make_commit("root", &[]),
        ];
        let nodes = compute_layout(&commits);
        assert!(nodes[0].is_merge); // M is a merge commit
        assert!(!nodes[1].is_merge);
    }
}
