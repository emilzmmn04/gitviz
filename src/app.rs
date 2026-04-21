use crate::git::model::{Commit, Refs};
use crate::graph::GraphNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filter,
}

pub struct App {
    pub commits: Vec<Commit>,
    pub refs: Refs,
    pub graph: Vec<GraphNode>,
    pub colors_enabled: bool,

    /// Indices into `commits` that pass the current filter (or all, when no filter).
    pub filtered: Vec<usize>,

    /// Index within `filtered` that is currently selected.
    pub selected: usize,

    /// Current input mode.
    pub mode: Mode,

    /// Current filter string (case-insensitive token match across commit metadata).
    pub filter: String,

    /// Whether the details panel is expanded (vs. collapsed).
    pub details_expanded: bool,
}

impl App {
    pub fn new(
        commits: Vec<Commit>,
        refs: Refs,
        graph: Vec<GraphNode>,
        colors_enabled: bool,
    ) -> Self {
        let filtered: Vec<usize> = (0..commits.len()).collect();
        App {
            commits,
            refs,
            graph,
            colors_enabled,
            filtered,
            selected: 0,
            mode: Mode::Normal,
            filter: String::new(),
            details_expanded: true,
        }
    }

    // ── Navigation ──────────────────────────────────────────────────────────

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_to_top(&mut self) {
        self.selected = 0;
    }

    pub fn move_to_bottom(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = self.filtered.len() - 1;
        }
    }

    // ── Details ─────────────────────────────────────────────────────────────

    pub fn toggle_details(&mut self) {
        self.details_expanded = !self.details_expanded;
    }

    // ── Filter ──────────────────────────────────────────────────────────────

    pub fn enter_filter_mode(&mut self) {
        self.mode = Mode::Filter;
    }

    pub fn exit_filter_mode(&mut self) {
        self.mode = Mode::Normal;
        self.filter.clear();
        self.recompute_filter();
    }

    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.recompute_filter();
    }

    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.recompute_filter();
    }

    pub fn replace_data(&mut self, commits: Vec<Commit>, refs: Refs, graph: Vec<GraphNode>) {
        let selected_oid = self.selected_commit().map(|commit| commit.oid.clone());
        self.commits = commits;
        self.refs = refs;
        self.graph = graph;
        self.recompute_filter();

        if let Some(selected_oid) = selected_oid {
            if let Some(position) = self
                .filtered
                .iter()
                .position(|&index| self.commits[index].oid == selected_oid)
            {
                self.selected = position;
            }
        }

        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }

    fn recompute_filter(&mut self) {
        let query = self.filter.to_lowercase();
        let tokens: Vec<&str> = query.split_whitespace().collect();

        if tokens.is_empty() {
            self.filtered = (0..self.commits.len()).collect();
        } else {
            self.filtered = self
                .commits
                .iter()
                .enumerate()
                .filter(|(_, commit)| {
                    let mut haystack = format!(
                        "{}\n{}\n{}\n{}",
                        commit.subject,
                        commit.author,
                        commit.author_email,
                        commit.oid
                    );

                    if !commit.body.is_empty() {
                        haystack.push('\n');
                        haystack.push_str(&commit.body);
                    }

                    let labels = self.refs.labels_for(&commit.oid);
                    if !labels.is_empty() {
                        haystack.push('\n');
                        haystack.push_str(&labels.join(" "));
                    }

                    let haystack = haystack.to_lowercase();
                    tokens.iter().all(|token| haystack.contains(token))
                })
                .map(|(i, _)| i)
                .collect();
        }
        // Clamp selection
        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// The currently selected commit, if any.
    pub fn selected_commit(&self) -> Option<&Commit> {
        self.filtered
            .get(self.selected)
            .and_then(|&i| self.commits.get(i))
    }
}
