use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::build_client;
use crate::api_models::ApiProject;
use crate::pagination::PaginatedResponse;

pub async fn list_projects(limit: u32, offset: u32) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects?limit={}&offset={}", cfg.api_base_url, limit, offset);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("プロジェクト一覧取得に失敗しました: {}", resp.status()));
    }
    let paginated: PaginatedResponse<ApiProject> = resp.json().await?;
    for p in paginated.data {
        println!("- {} ({})", p.name, p.slug);
    }
    println!("取得件数: {} (limit={}, offset={})", paginated.meta.count, paginated.meta.limit, paginated.meta.offset);
    Ok(())
}

pub async fn get_project(slug: &str) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}", cfg.api_base_url, slug);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("プロジェクト取得に失敗しました: {}", resp.status()));
    }
    let project: ApiProject = resp.json().await?;
    println!("プロジェクト: {} ({})", project.name, project.slug);
    println!("説明: {}", project.description.unwrap_or_default());
    println!("ダウンロード数: {}", project.downloads.total);
    Ok(())
}
