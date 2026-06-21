use clap::{Parser, Subcommand};
mod api_client;
mod api_models;
mod commands;
mod config;
mod pagination;

#[derive(Parser)]
#[command(name = "modparks", version = "0.1.0", author = "Pitan76", about = "ModParks CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save API key to config (login)
    Login { api_key: String },
    /// List projects
    Projects {
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
        #[arg(short, long, default_value_t = 0)]
        offset: u32,
    },
    /// Get a specific project by slug
    Project { slug: String },
    /// List versions of a project
    Versions { slug: String, #[arg(short, long, default_value_t = 20)] limit: u32 },
    /// List ideas
    Ideas { #[arg(short, long, default_value_t = 20)] limit: u32 },
    /// List comments of a project
    Comments { slug: String },
    /// Post a comment to a project
    CommentPost { slug: String, content: String },
    /// Sync a project with external platform
    Sync { slug: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Login { api_key } => commands::login(&api_key).await?,
        Commands::Projects { limit, offset } => commands::list_projects(limit, offset).await?,
        Commands::Project { slug } => commands::get_project(&slug).await?,
        Commands::Versions { slug, limit } => commands::list_versions(&slug, limit).await?,
        Commands::Ideas { limit } => commands::list_ideas(limit).await?,
        Commands::Comments { slug } => commands::list_comments(&slug).await?,
        Commands::CommentPost { slug, content } => commands::post_comment(&slug, &content).await?,
        Commands::Sync { slug } => commands::sync_project(&slug).await?,
    }
    Ok(())
}
