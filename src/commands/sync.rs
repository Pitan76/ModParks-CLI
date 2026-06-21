// src/commands/sync.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::build_client;

pub async fn sync_project(slug: &str) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/sync", cfg.api_base_url, slug);
    let resp = client.post(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("同期に失敗しました: {}", resp.status()));
    }
    println!("プロジェクト '{}' を同期しました。", slug);
    Ok(())
}
