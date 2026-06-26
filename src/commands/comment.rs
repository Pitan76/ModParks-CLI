// src/commands/comment.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::{build_client, cached_get};
use crate::api_models::ApiComment;
use crate::pagination::PaginatedResponse;

pub async fn list_comments(slug: &str) -> Result<()> {
    let cfg = Config::load()?;
    let url = format!("{}/projects/{}/comments", cfg.api_base_url, slug);
    let paginated: PaginatedResponse<ApiComment> = cached_get(&url, &cfg).await?;
    for comment in paginated.data {
        let author = comment.author
            .as_ref()
            .and_then(|a| a.display_name.as_deref().or(Some(a.username.as_str())))
            .unwrap_or("不明");
        println!("[{}] {}: {}", comment.id, author, comment.content);
    }
    println!("取得件数: {}", paginated.meta.count);
    Ok(())
}

pub async fn post_comment(slug: &str, content: &str) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/comments", cfg.api_base_url, slug);
    let body = serde_json::json!({ "content": content });
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("コメント投稿に失敗しました: {}", resp.status()));
    }
    println!("コメントを投稿しました。");
    Ok(())
}
