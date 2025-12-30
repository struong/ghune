use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub stargazers_count: u32,
    pub language: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub html_url: String,
}

impl From<octocrab::models::Repository> for Repository {
    fn from(repo: octocrab::models::Repository) -> Self {
        Self {
            id: repo.id.0,
            name: repo.name,
            full_name: repo.full_name.unwrap_or_default(),
            description: repo.description,
            private: repo.private.unwrap_or(false),
            fork: repo.fork.unwrap_or(false),
            archived: repo.archived.unwrap_or(false),
            stargazers_count: repo.stargazers_count.unwrap_or(0),
            language: repo.language.and_then(|v| v.as_str().map(String::from)),
            created_at: repo.created_at.map(|d| d.to_string()),
            updated_at: repo.updated_at.map(|d| d.to_string()),
            html_url: repo.html_url.map(|u| u.to_string()).unwrap_or_default(),
        }
    }
}
