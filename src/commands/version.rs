// src/commands/version.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::build_client;
use crate::api_models::ApiProject; // placeholder, replace with proper Version struct later
use reqwest::Client;

/// List versions of a project.
/// Currently parses the response as a generic JSON array and prints each entry.
pub async fn list_versions(slug: &str, limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/versions?limit={}", cfg.api_base_url, slug, limit);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("バージョン取得に失敗しました: {}", resp.status()));
    }
    // For now, treat response as generic JSON and pretty print.
    let json: serde_json::Value = resp.json().await?;
    println!("バージョン情報: {}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
