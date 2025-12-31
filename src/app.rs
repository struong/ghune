use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    fuzzy::FuzzyMatcher,
    github::GitHubClient,
    state::{AppMode, AppState, SortMode, StatusLevel},
    tui::{self, Event, EventHandler},
};

pub enum Action {
    None,
    Quit,
    Refresh,
    ExecuteDeletion,
}

pub struct App {
    pub state: AppState,
    fuzzy: FuzzyMatcher,
    client: GitHubClient,
}

impl App {
    pub async fn new(token: &str, dry_run: bool) -> Result<Self> {
        let client = GitHubClient::new(token).await?;
        Ok(Self {
            state: AppState::new(dry_run),
            fuzzy: FuzzyMatcher::new(),
            client,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = tui::terminal::init()?;
        let mut events = EventHandler::new(Duration::from_millis(100));

        self.load_repos().await;

        loop {
            terminal.draw(|frame| tui::ui::render(frame, &self.state))?;

            match events.next().await? {
                Event::Key(key) => {
                    let action = self.handle_key(key);
                    match action {
                        Action::Quit => break,
                        Action::Refresh => {
                            self.state.loading = true;
                            self.load_repos().await;
                        }
                        Action::ExecuteDeletion => {
                            self.execute_deletion(&mut terminal).await;
                        }
                        Action::None => {}
                    }
                }
                Event::Tick => {
                    self.state.clear_status();
                }
                Event::Resize => {}
            }
        }

        tui::terminal::restore()?;
        Ok(())
    }

    async fn load_repos(&mut self) {
        self.state.loading = true;
        match self.client.list_repos().await {
            Ok(repos) => {
                self.state.repositories = repos;
                self.update_filtered();
                self.state.loading = false;
                self.state.set_status(
                    format!("Loaded {} repositories", self.state.repositories.len()),
                    StatusLevel::Success,
                );
            }
            Err(e) => {
                self.state.loading = false;
                self.state
                    .set_status(format!("Failed to load repos: {}", e), StatusLevel::Error);
            }
        }
    }

    fn update_filtered(&mut self) {
        let mut indices = self
            .fuzzy
            .filter(&self.state.repositories, &self.state.search_query);

        // Apply filters
        indices.retain(|&idx| {
            let repo = &self.state.repositories[idx];
            if self.state.filter_private && !repo.private {
                return false;
            }
            if self.state.filter_forks && !repo.fork {
                return false;
            }
            true
        });

        // Apply sorting
        match self.state.sort_mode {
            SortMode::LastUpdated => {
                indices.sort_by(|&a, &b| {
                    let repo_a = &self.state.repositories[a];
                    let repo_b = &self.state.repositories[b];
                    repo_b.updated_at.cmp(&repo_a.updated_at)
                });
            }
            SortMode::Created => {
                indices.sort_by(|&a, &b| {
                    let repo_a = &self.state.repositories[a];
                    let repo_b = &self.state.repositories[b];
                    repo_b.created_at.cmp(&repo_a.created_at)
                });
            }
            SortMode::Name => {
                indices.sort_by(|&a, &b| {
                    let repo_a = &self.state.repositories[a];
                    let repo_b = &self.state.repositories[b];
                    repo_a
                        .full_name
                        .to_lowercase()
                        .cmp(&repo_b.full_name.to_lowercase())
                });
            }
            SortMode::Stars => {
                indices.sort_by(|&a, &b| {
                    let repo_a = &self.state.repositories[a];
                    let repo_b = &self.state.repositories[b];
                    repo_b.stargazers_count.cmp(&repo_a.stargazers_count)
                });
            }
        }

        self.state.filtered_indices = indices;
        if self.state.selected_index >= self.state.filtered_indices.len() {
            self.state.selected_index = 0;
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match self.state.mode {
            AppMode::Search => self.handle_search_key(key),
            AppMode::Staging => self.handle_staging_key(key),
            AppMode::Deleting => Action::None,
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,

            (KeyCode::Char('j'), KeyModifiers::CONTROL) | (KeyCode::Down, _) => {
                self.state.move_selection(1);
                Action::None
            }

            (KeyCode::Char('k'), KeyModifiers::CONTROL) | (KeyCode::Up, _) => {
                self.state.move_selection(-1);
                Action::None
            }

            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.state.toggle_stage();
                self.state.move_selection(1);
                Action::None
            }

            (KeyCode::Tab, KeyModifiers::NONE) => {
                if !self.state.staged_for_deletion.is_empty() {
                    self.state.mode = AppMode::Staging;
                    self.state.staged_selected_index = 0;
                } else {
                    self.state.set_status(
                        "No repos staged for deletion".to_string(),
                        StatusLevel::Warning,
                    );
                }
                Action::None
            }

            (KeyCode::Char('r'), KeyModifiers::CONTROL) => Action::Refresh,

            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                self.state.filter_private = !self.state.filter_private;
                let status = if self.state.filter_private {
                    "Filter: private repos only"
                } else {
                    "Filter: showing all repos"
                };
                self.state.set_status(status.to_string(), StatusLevel::Info);
                self.update_filtered();
                Action::None
            }

            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.state.filter_forks = !self.state.filter_forks;
                let status = if self.state.filter_forks {
                    "Filter: forks only"
                } else {
                    "Filter: showing all repos"
                };
                self.state.set_status(status.to_string(), StatusLevel::Info);
                self.update_filtered();
                Action::None
            }

            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.state.sort_mode = self.state.sort_mode.next();
                self.state.set_status(
                    format!("Sort: {}", self.state.sort_mode.label()),
                    StatusLevel::Info,
                );
                self.update_filtered();
                Action::None
            }

            (KeyCode::Esc, _) => {
                self.state.search_query.clear();
                self.update_filtered();
                Action::None
            }

            (KeyCode::Backspace, _) => {
                self.state.search_query.pop();
                self.update_filtered();
                Action::None
            }

            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.state.search_query.push(c);
                self.update_filtered();
                Action::None
            }

            _ => Action::None,
        }
    }

    fn handle_staging_key(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,

            (KeyCode::Char('j'), KeyModifiers::CONTROL) | (KeyCode::Down, _) => {
                self.state.move_selection(1);
                Action::None
            }

            (KeyCode::Char('k'), KeyModifiers::CONTROL) | (KeyCode::Up, _) => {
                self.state.move_selection(-1);
                Action::None
            }

            (KeyCode::Backspace, _) | (KeyCode::Delete, _) => {
                self.state.unstage_selected();
                if self.state.staged_for_deletion.is_empty() {
                    self.state.mode = AppMode::Search;
                }
                Action::None
            }

            (KeyCode::Enter, KeyModifiers::NONE) => {
                if !self.state.staged_for_deletion.is_empty() {
                    Action::ExecuteDeletion
                } else {
                    self.state.mode = AppMode::Search;
                    Action::None
                }
            }

            (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Esc, _) => {
                self.state.mode = AppMode::Search;
                Action::None
            }

            _ => Action::None,
        }
    }

    async fn execute_deletion(&mut self, terminal: &mut tui::terminal::Terminal) {
        self.state.mode = AppMode::Deleting;
        let repos_to_delete: Vec<String> = self.state.staged_for_deletion.iter().cloned().collect();
        let total = repos_to_delete.len();

        let mut deleted = Vec::new();
        let mut failed = Vec::new();

        for (i, repo_name) in repos_to_delete.iter().enumerate() {
            self.state.deletion_progress = Some((i + 1, total));
            self.state.deleting_repo = Some(repo_name.clone());

            // Redraw to show progress
            let _ = terminal.draw(|frame| tui::ui::render(frame, &self.state));

            if self.state.dry_run {
                // Animate spinner during dry run
                for _ in 0..5 {
                    self.state.advance_spinner();
                    let _ = terminal.draw(|frame| tui::ui::render(frame, &self.state));
                    tokio::time::sleep(Duration::from_millis(80)).await;
                }
                deleted.push(repo_name.clone());
            } else {
                // Start spinner animation in background
                let delete_result = self.client.delete_repo(repo_name).await;
                match delete_result {
                    Ok(()) => {
                        deleted.push(repo_name.clone());
                    }
                    Err(e) => {
                        failed.push((repo_name.clone(), e.to_string()));
                    }
                }
            }
        }

        for name in &deleted {
            self.state.staged_for_deletion.remove(name);
            self.state.repositories.retain(|r| &r.full_name != name);
        }

        self.update_filtered();
        self.state.deletion_progress = None;
        self.state.deleting_repo = None;
        self.state.mode = AppMode::Search;

        if failed.is_empty() {
            let msg = if self.state.dry_run {
                format!("[DRY RUN] Would have deleted {} repos", deleted.len())
            } else {
                format!("Successfully deleted {} repos", deleted.len())
            };
            self.state.set_status(msg, StatusLevel::Success);
        } else {
            self.state.set_status(
                format!(
                    "Deleted {}, failed {}: {}",
                    deleted.len(),
                    failed.len(),
                    failed.first().map(|(_, e)| e.as_str()).unwrap_or("")
                ),
                StatusLevel::Error,
            );
        }
    }
}
