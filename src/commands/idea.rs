// src/commands/idea.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::cached_get;
use crate::api_models::ApiIdea;
use crate::pagination::PaginatedResponse;

pub async fn list_ideas(limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let url = format!("{}/ideas?limit={}", cfg.api_base_url, limit);
    let paginated: PaginatedResponse<ApiIdea> = cached_get(&url, &cfg).await?;
    for idea in paginated.data {
        println!("[{}] {} ({})", idea.status, idea.title, idea.id);
    }
    println!("取得件数: {} (limit={})", paginated.meta.count, paginated.meta.limit);
    Ok(())
}

pub async fn get_idea(id: &str) -> Result<()> {
    let cfg = Config::load()?;
    let url = format!("{}/ideas/{}", cfg.api_base_url, id);
    #[derive(serde::Deserialize, serde::Serialize)]
    struct Wrapper { data: ApiIdea }
    let wrapper: Wrapper = cached_get(&url, &cfg).await
        .map_err(|_| anyhow!("アイデア取得に失敗しました"))?;
    let idea = wrapper.data;
    println!("ID:     {}", idea.id);
    println!("タイトル: {}", idea.title);
    println!("状態:   {}", idea.status);
    println!("内容:\n{}", idea.content);
    Ok(())
}

