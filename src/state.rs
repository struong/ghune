use std::collections::HashSet;

use crate::github::types::Repository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Search,
    Staging,
    Deleting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    LastUpdated,
    Created,
    Name,
    Stars,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::LastUpdated => SortMode::Created,
            SortMode::Created => SortMode::Name,
            SortMode::Name => SortMode::Stars,
            SortMode::Stars => SortMode::LastUpdated,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortMode::LastUpdated => "Updated",
            SortMode::Created => "Created",
            SortMode::Name => "Name",
            SortMode::Stars => "Stars",
        }
    }
}

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct AppState {
    pub mode: AppMode,
    pub repositories: Vec<Repository>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    pub staged_for_deletion: HashSet<String>,
    pub staged_selected_index: usize,
    pub search_query: String,
    pub status_message: Option<(String, StatusLevel)>,
    pub loading: bool,
    pub deletion_progress: Option<(usize, usize)>,
    pub deleting_repo: Option<String>,
    pub spinner_frame: usize,
    pub dry_run: bool,
    pub filter_private: bool,
    pub filter_forks: bool,
    pub sort_mode: SortMode,
}

impl AppState {
    pub fn new(dry_run: bool) -> Self {
        Self {
            mode: AppMode::Search,
            repositories: Vec::new(),
            filtered_indices: Vec::new(),
            selected_index: 0,
            staged_for_deletion: HashSet::new(),
            staged_selected_index: 0,
            search_query: String::new(),
            status_message: None,
            loading: true,
            deletion_progress: None,
            deleting_repo: None,
            spinner_frame: 0,
            dry_run,
            filter_private: false,
            filter_forks: false,
            sort_mode: SortMode::default(),
        }
    }

    pub fn advance_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn spinner(&self) -> &'static str {
        SPINNER_FRAMES[self.spinner_frame]
    }

    pub fn selected_repo(&self) -> Option<&Repository> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.repositories.get(idx))
    }

    pub fn staged_repos(&self) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|r| self.staged_for_deletion.contains(&r.full_name))
            .collect()
    }

    pub fn staged_repos_sorted(&self) -> Vec<String> {
        let mut names: Vec<_> = self.staged_for_deletion.iter().cloned().collect();
        names.sort();
        names
    }

    pub fn move_selection(&mut self, delta: i32) {
        match self.mode {
            AppMode::Search => {
                if self.filtered_indices.is_empty() {
                    return;
                }
                let len = self.filtered_indices.len() as i32;
                let new_idx = (self.selected_index as i32 + delta).rem_euclid(len);
                self.selected_index = new_idx as usize;
            }
            AppMode::Staging => {
                if self.staged_for_deletion.is_empty() {
                    return;
                }
                let len = self.staged_for_deletion.len() as i32;
                let new_idx = (self.staged_selected_index as i32 + delta).rem_euclid(len);
                self.staged_selected_index = new_idx as usize;
            }
            AppMode::Deleting => {}
        }
    }

    pub fn toggle_stage(&mut self) {
        if let Some(repo) = self.selected_repo() {
            let name = repo.full_name.clone();
            if self.staged_for_deletion.contains(&name) {
                self.staged_for_deletion.remove(&name);
            } else {
                self.staged_for_deletion.insert(name);
            }
        }
    }

    pub fn unstage_selected(&mut self) {
        let names = self.staged_repos_sorted();
        if let Some(name) = names.get(self.staged_selected_index) {
            self.staged_for_deletion.remove(name);
            if self.staged_selected_index > 0 && self.staged_selected_index >= self.staged_for_deletion.len() {
                self.staged_selected_index = self.staged_for_deletion.len().saturating_sub(1);
            }
        }
    }

    pub fn set_status(&mut self, message: String, level: StatusLevel) {
        self.status_message = Some((message, level));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}
