// src/commands/version.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::build_client;

/// List versions of a project.
pub async fn list_versions(slug: &str, limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/versions?limit={}", cfg.api_base_url, slug, limit);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("バージョン取得に失敗しました: {}", resp.status()));
    }
    let json: serde_json::Value = resp.json().await?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
