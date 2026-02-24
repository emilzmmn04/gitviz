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

    /// Indices into `commits` that pass the current filter (or all, when no filter).
    pub filtered: Vec<usize>,

    /// Index within `filtered` that is currently selected.
    pub selected: usize,

    /// Current input mode.
    pub mode: Mode,

    /// Current filter string (substring match on subject, case-insensitive).
    pub filter: String,

    /// Whether the details panel is expanded (vs. collapsed).
    pub details_expanded: bool,
}

impl App {
    pub fn new(commits: Vec<Commit>, refs: Refs, graph: Vec<GraphNode>) -> Self {
        let filtered: Vec<usize> = (0..commits.len()).collect();
        App {
            commits,
            refs,
            graph,
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

    fn recompute_filter(&mut self) {
        let needle = self.filter.to_lowercase();
        if needle.is_empty() {
            self.filtered = (0..self.commits.len()).collect();
        } else {
            self.filtered = self
                .commits
                .iter()
                .enumerate()
                .filter(|(_, c)| c.subject.to_lowercase().contains(&needle))
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
