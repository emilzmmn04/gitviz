use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::git::model::{Commit, CommitInspectData, InspectCacheEntry, Refs};
use crate::graph::GraphRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailsTab {
    Summary,
    Files,
    Diff,
}

impl DetailsTab {
    pub fn title(self) -> &'static str {
        match self {
            DetailsTab::Summary => "Summary",
            DetailsTab::Files => "Files",
            DetailsTab::Diff => "Diff",
        }
    }

    pub fn next(self) -> Self {
        match self {
            DetailsTab::Summary => DetailsTab::Files,
            DetailsTab::Files => DetailsTab::Diff,
            DetailsTab::Diff => DetailsTab::Summary,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            DetailsTab::Summary => DetailsTab::Diff,
            DetailsTab::Files => DetailsTab::Summary,
            DetailsTab::Diff => DetailsTab::Files,
        }
    }
}

pub struct App {
    pub commits: Vec<Commit>,
    pub refs: Refs,
    pub graph: Vec<GraphRow>,
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

    pub active_tab: DetailsTab,
    pub details_scroll: u16,
    pub help_open: bool,
    pub inspect_cache: HashMap<String, InspectCacheEntry>,
    pub status_message: Option<String>,
    pub status_deadline: Option<Instant>,
}

impl App {
    pub fn new(
        commits: Vec<Commit>,
        refs: Refs,
        graph: Vec<GraphRow>,
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
            active_tab: DetailsTab::Summary,
            details_scroll: 0,
            help_open: false,
            inspect_cache: HashMap::new(),
            status_message: None,
            status_deadline: None,
        }
    }

    // Navigation
    pub fn move_down(&mut self) -> bool {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
            self.details_scroll = 0;
            return true;
        }
        false
    }

    pub fn move_up(&mut self) -> bool {
        if self.selected > 0 {
            self.selected -= 1;
            self.details_scroll = 0;
            return true;
        }
        false
    }

    pub fn move_to_top(&mut self) -> bool {
        if self.selected != 0 {
            self.selected = 0;
            self.details_scroll = 0;
            return true;
        }
        false
    }

    pub fn move_to_bottom(&mut self) -> bool {
        if !self.filtered.is_empty() {
            let new_selected = self.filtered.len() - 1;
            if self.selected != new_selected {
                self.selected = new_selected;
                self.details_scroll = 0;
                return true;
            }
        }
        false
    }

    pub fn search_next(&mut self) -> bool {
        if self.filter.is_empty() || self.filtered.is_empty() {
            return false;
        }
        self.selected = (self.selected + 1) % self.filtered.len();
        self.details_scroll = 0;
        true
    }

    pub fn search_previous(&mut self) -> bool {
        if self.filter.is_empty() || self.filtered.is_empty() {
            return false;
        }
        self.selected = if self.selected == 0 {
            self.filtered.len() - 1
        } else {
            self.selected - 1
        };
        self.details_scroll = 0;
        true
    }

    // Details
    pub fn toggle_details(&mut self) {
        self.details_expanded = !self.details_expanded;
        self.details_scroll = 0;
    }

    pub fn cycle_tab_forward(&mut self) {
        self.active_tab = self.active_tab.next();
        self.details_scroll = 0;
        self.prepare_selected_inspect_retry();
    }

    pub fn cycle_tab_backward(&mut self) {
        self.active_tab = self.active_tab.previous();
        self.details_scroll = 0;
        self.prepare_selected_inspect_retry();
    }

    pub fn scroll_details_lines(&mut self, amount: i16) {
        if amount >= 0 {
            self.details_scroll = self.details_scroll.saturating_add(amount as u16);
        } else {
            self.details_scroll = self.details_scroll.saturating_sub((-amount) as u16);
        }
    }

    pub fn clamp_details_scroll(&mut self, max_scroll: u16) {
        if self.details_scroll > max_scroll {
            self.details_scroll = max_scroll;
        }
    }

    // Help overlay
    pub fn toggle_help(&mut self) {
        self.help_open = !self.help_open;
    }

    pub fn close_help(&mut self) {
        self.help_open = false;
    }

    // Filter
    pub fn enter_filter_mode(&mut self) {
        self.mode = Mode::Filter;
    }

    pub fn exit_filter_mode(&mut self) {
        self.mode = Mode::Normal;
        self.filter.clear();
        self.recompute_filter();
    }

    pub fn confirm_filter(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.recompute_filter();
    }

    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.recompute_filter();
    }

    pub fn replace_data(&mut self, commits: Vec<Commit>, refs: Refs, graph: Vec<GraphRow>) {
        let selected_oid = self.selected_commit().map(|commit| commit.oid.clone());
        self.commits = commits;
        self.refs = refs;
        self.graph = graph;
        self.inspect_cache.clear();
        self.active_tab = DetailsTab::Summary;
        self.details_scroll = 0;
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

    pub fn selected_commit(&self) -> Option<&Commit> {
        self.filtered
            .get(self.selected)
            .and_then(|&i| self.commits.get(i))
    }

    pub fn selected_commit_oid(&self) -> Option<&str> {
        self.selected_commit().map(|commit| commit.oid.as_str())
    }

    pub fn should_load_selected_inspect(&self) -> bool {
        if self.help_open || matches!(self.active_tab, DetailsTab::Summary) {
            return false;
        }

        let Some(oid) = self.selected_commit_oid() else {
            return false;
        };

        self.inspect_cache.get(oid).is_none()
    }

    pub fn insert_loading_for_selected(&mut self) -> Option<String> {
        let oid = self.selected_commit_oid()?.to_string();
        self.inspect_cache
            .insert(oid.clone(), InspectCacheEntry::Loading);
        Some(oid)
    }

    pub fn selected_inspect_data(&self) -> Option<&CommitInspectData> {
        let oid = self.selected_commit_oid()?;
        match self.inspect_cache.get(oid) {
            Some(InspectCacheEntry::Ready(data)) => Some(data),
            _ => None,
        }
    }

    pub fn selected_inspect_error(&self) -> Option<&str> {
        let oid = self.selected_commit_oid()?;
        match self.inspect_cache.get(oid) {
            Some(InspectCacheEntry::Error(message)) => Some(message.as_str()),
            _ => None,
        }
    }

    pub fn cache_inspect_ready(&mut self, oid: String, data: CommitInspectData) {
        self.inspect_cache.insert(oid, InspectCacheEntry::Ready(data));
    }

    pub fn cache_inspect_error(&mut self, oid: String, message: String) {
        self.inspect_cache.insert(oid, InspectCacheEntry::Error(message));
    }

    fn prepare_selected_inspect_retry(&mut self) {
        if matches!(self.active_tab, DetailsTab::Summary) {
            return;
        }

        let Some(oid) = self.selected_commit_oid().map(str::to_string) else {
            return;
        };

        if matches!(self.inspect_cache.get(&oid), Some(InspectCacheEntry::Error(_))) {
            self.inspect_cache.remove(&oid);
        }
    }

    pub fn set_status<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some(message.into());
        self.status_deadline = Some(Instant::now() + Duration::from_secs(3));
    }

    pub fn clear_expired_status(&mut self) {
        if let Some(deadline) = self.status_deadline {
            if Instant::now() >= deadline {
                self.status_message = None;
                self.status_deadline = None;
            }
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
                        commit.subject, commit.author, commit.author_email, commit.oid
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

        self.details_scroll = 0;

        if self.filtered.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len() - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::model::Commit;

    fn commit(oid: &str, subject: &str) -> Commit {
        Commit {
            oid: oid.to_string(),
            parents: Vec::new(),
            author: "Author".to_string(),
            author_email: "author@example.com".to_string(),
            timestamp: 0,
            subject: subject.to_string(),
            body: String::new(),
        }
    }

    fn app_with_commits(commits: Vec<Commit>) -> App {
        let graph = Vec::new();
        App::new(commits, Refs::default(), graph, true)
    }

    #[test]
    fn test_tab_cycling() {
        let mut app = app_with_commits(vec![commit("a", "first")]);
        assert_eq!(app.active_tab, DetailsTab::Summary);
        app.cycle_tab_forward();
        assert_eq!(app.active_tab, DetailsTab::Files);
        app.cycle_tab_forward();
        assert_eq!(app.active_tab, DetailsTab::Diff);
        app.cycle_tab_forward();
        assert_eq!(app.active_tab, DetailsTab::Summary);
        app.cycle_tab_backward();
        assert_eq!(app.active_tab, DetailsTab::Diff);
    }

    #[test]
    fn test_search_wraparound() {
        let mut app = app_with_commits(vec![
            commit("a", "fix first"),
            commit("b", "fix second"),
            commit("c", "fix third"),
        ]);
        app.filter = "fix".to_string();
        app.filter_push(' ');
        app.filter_pop();
        app.selected = 2;
        assert!(app.search_next());
        assert_eq!(app.selected, 0);
        assert!(app.search_previous());
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn test_details_scroll_clamping() {
        let mut app = app_with_commits(vec![commit("a", "first")]);
        app.scroll_details_lines(50);
        assert_eq!(app.details_scroll, 50);
        app.clamp_details_scroll(12);
        assert_eq!(app.details_scroll, 12);
        app.scroll_details_lines(-20);
        assert_eq!(app.details_scroll, 0);
    }
}
