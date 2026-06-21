// src/commands/idea.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::build_client;
use crate::api_models::ApiIdea;
use crate::pagination::PaginatedResponse;

pub async fn list_ideas(limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/ideas?limit={}", cfg.api_base_url, limit);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("アイデア一覧取得に失敗しました: {}", resp.status()));
    }
    let paginated: PaginatedResponse<ApiIdea> = resp.json().await?;
    for idea in paginated.data {
        println!("[{}] {} ({})", idea.status, idea.title, idea.id);
    }
    println!("取得件数: {} (limit={})", paginated.meta.count, paginated.meta.limit);
    Ok(())
}
