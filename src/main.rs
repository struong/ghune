mod app;
mod auth;
mod fuzzy;
mod github;
mod state;
mod tui;

use clap::Parser;
use color_eyre::eyre::Result;

use app::App;
use auth::TokenManager;

#[derive(Parser)]
#[command(name = "prune")]
#[command(about = "Interactive GitHub repository deletion tool")]
#[command(version)]
struct Cli {
    /// Run without actually deleting repositories
    #[arg(long)]
    dry_run: bool,

    /// Clear stored GitHub token
    #[arg(long)]
    logout: bool,

    /// Show only forked repositories
    #[arg(long)]
    forks_only: bool,

    /// Show only private repositories
    #[arg(long)]
    private_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let token_manager = TokenManager::new()?;

    if cli.logout {
        token_manager.clear_token()?;
        println!("GitHub token cleared.");
        return Ok(());
    }

    let token = token_manager.get_or_prompt_token()?;

    let mut app = App::new(&token, cli.dry_run).await?;
    app.run().await
}
