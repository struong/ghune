use color_eyre::eyre::{eyre, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn token_file_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| eyre!("Could not find config directory"))?
        .join("ghune");
    Ok(config_dir.join("token"))
}

pub struct TokenManager;

impl TokenManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn get_token(&self) -> Result<Option<String>> {
        let path = token_file_path()?;
        if path.exists() {
            let token = fs::read_to_string(&path)?.trim().to_string();
            if !token.is_empty() {
                return Ok(Some(token));
            }
        }
        Ok(None)
    }

    pub fn get_or_prompt_token(&self) -> Result<String> {
        if let Some(token) = self.get_token()? {
            return Ok(token);
        }

        let token = self.prompt_for_token()?;
        self.store_token(&token)?;
        Ok(token)
    }

    fn prompt_for_token(&self) -> Result<String> {
        eprintln!("GitHub personal access token not found.");
        eprintln!();
        eprintln!("Create one at: https://github.com/settings/tokens/new");
        eprintln!("Required scopes: 'delete_repo' and 'repo' (for private repos)");
        eprintln!();
        eprint!("Enter token: ");
        io::stderr().flush()?;

        let token = rpassword::read_password()?;

        if token.is_empty() {
            return Err(eyre!("Token cannot be empty"));
        }

        Ok(token)
    }

    pub fn store_token(&self, token: &str) -> Result<()> {
        let path = token_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, token)?;

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    pub fn clear_token(&self) -> Result<()> {
        let path = token_file_path()?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}
