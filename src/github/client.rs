use color_eyre::eyre::Result;
use octocrab::Octocrab;

use super::types::Repository;

pub struct GitHubClient {
    octocrab: Octocrab,
}

impl GitHubClient {
    pub async fn new(token: &str) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()?;

        Ok(Self { octocrab })
    }

    pub async fn list_repos(&self) -> Result<Vec<Repository>> {
        let mut repos = Vec::new();
        let mut page = 1u32;

        loop {
            let response: Vec<octocrab::models::Repository> = self
                .octocrab
                .get(
                    "/user/repos",
                    Some(&[
                        ("per_page", "100"),
                        ("page", &page.to_string()),
                        ("affiliation", "owner"),
                        ("sort", "updated"),
                    ]),
                )
                .await?;

            if response.is_empty() {
                break;
            }

            repos.extend(response.into_iter().map(Repository::from));
            page += 1;
        }

        Ok(repos)
    }

    pub async fn delete_repo(&self, full_name: &str) -> Result<()> {
        let url = format!("/repos/{}", full_name);
        self.octocrab._delete(&url, None::<&()>).await?;
        Ok(())
    }
}
