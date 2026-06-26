use clap::{Parser, CommandFactory};
mod args;
use args::{Cli, Commands};

mod api_client;
mod api_models;
mod cache;
mod commands;
mod config;
mod pagination;
mod tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Login { api_key } => commands::login(&api_key).await?,
            Commands::Update => commands::update()?,
            Commands::Logout => {
                let mut cfg = config::Config::load()?;
                cfg.api_key = String::new();
                cfg.save()?;
                println!("ログアウトしました (API キーを削除しました)。");
            }
            Commands::Profile { username } => commands::display_profile(username.as_deref()).await?,
            Commands::MyProjects { page, limit } => {
                let offset = page.saturating_sub(1) * limit;
                commands::list_my_projects(limit, offset).await?
            }
            Commands::Projects { page, page_opt, limit } => {
                let p = page_opt.unwrap_or(page);
                let offset = p.saturating_sub(1) * limit;
                commands::list_projects(limit, offset).await?
            }
            Commands::Project { slug } => commands::get_project(&slug).await?,
            Commands::ProjectCreate { name, slug, description, project_type } => {
                commands::create_project(name, slug, description, project_type).await?
            }
            Commands::ProjectEdit { slug, name, new_slug, description, project_type } => {
                commands::update_project(slug, name, new_slug, description, project_type).await?
            }
            Commands::Versions { slug, limit } => commands::list_versions(&slug, limit).await?,
            Commands::VersionCreate { slug, file, url, file_name, version_number, loaders, mc_versions, changelog } => {
                commands::create_version(slug, file, url, file_name, version_number, loaders, mc_versions, changelog).await?
            }
            Commands::Download { slug, version, out } => commands::download_version(slug, version, out).await?,
            Commands::Ideas { limit } => commands::list_ideas(limit).await?,
            Commands::Idea { id } => commands::get_idea(&id).await?,
            Commands::Comments { slug } => commands::list_comments(&slug).await?,
            Commands::CommentPost { slug, content } => commands::post_comment(&slug, &content).await?,
            Commands::Sync { slug } => commands::sync_project(&slug).await?,
            Commands::Tui => tui::run_tui().await?,
            Commands::CacheClear => {
                let n = cache::clear()?;
                println!("{}件のキャッシュを削除しました。", n);
            }
        }
    } else {
        // 実行ファイル名を確認
        let exe_name = std::env::args().next().unwrap_or_default().to_lowercase();
        if exe_name.contains("modparks-tui") {
            // エイリアス (modparks-tui) 経由で呼ばれた場合のみTUIを起動
            tui::run_tui().await?;
        } else {
            // modparks-cli で引数なしの場合はヘルプを表示
            Cli::command().print_help()?;
        }
    }
    Ok(())
}