use color_eyre::eyre::{eyre, Result};
use keyring::Entry;
use std::io::{self, Write};

const SERVICE_NAME: &str = "ghune-github-cli";
const USERNAME: &str = "github-token";

pub struct TokenManager {
    entry: Entry,
}

impl TokenManager {
    pub fn new() -> Result<Self> {
        let entry = Entry::new(SERVICE_NAME, USERNAME)?;
        Ok(Self { entry })
    }

    pub fn get_token(&self) -> Result<Option<String>> {
        match self.entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
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
        self.entry.set_password(token)?;
        Ok(())
    }

    pub fn clear_token(&self) -> Result<()> {
        match self.entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
