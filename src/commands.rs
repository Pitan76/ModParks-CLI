// src/commands.rs
use anyhow::{anyhow, Result};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use crate::config::Config;

#[derive(Debug, Deserialize, Serialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    meta: Meta,
}

#[derive(Debug, Deserialize, Serialize)]
struct Meta {
    limit: u32,
    offset: u32,
    count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiProject {
    id: String,
    slug: String,
    name: String,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    #[serde(rename = "type")]
    project_type: String,
    license: String,
    downloads: Downloads,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
    author: Option<Author>,
    categories: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiProjectDetail {
    id: String,
    slug: String,
    name: String,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    #[serde(rename = "type")]
    project_type: String,
    license: String,
    downloads: Downloads,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
    author: Author,
    tags: Vec<String>,
    dependencies: Vec<Dependency>,
    dependents: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Downloads {
    total: u32,
    native: u32,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Author {
    username: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "avatarUrl")]
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Dependency {
    id: String,
    #[serde(rename = "dependencyType")]
    dependency_type: String,
    project: DependencyProject,
}

#[derive(Debug, Deserialize, Serialize)]
struct DependencyProject {
    id: String,
    slug: String,
    name: String,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiIdea {
    id: String,
    title: String,
    content: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: i64,
    #[serde(rename = "updatedAt")]
    updated_at: i64,
    author: Option<Author>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiComment {
    id: String,
    content: String,
    author: Option<Author>,
    #[serde(rename = "createdAt")]
    created_at: i64,
}

fn build_client(api_key: &str) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    let auth_val = format!("Bearer {}", api_key);
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&auth_val)?);
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

pub async fn login(api_key: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    cfg.api_key = api_key.to_string();
    cfg.save()?;
    println!("✅ API キーを保存しました。");
    Ok(())
}

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

pub async fn list_versions(slug: &str, limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/versions?limit={}", cfg.api_base_url, slug, limit);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("バージョン取得に失敗しました: {}", resp.status()));
    }
    let paginated: PaginatedResponse<serde_json::Value> = resp.json().await?; // raw JSON for flexibility
    for v in paginated.data {
        println!("- {}", v.get("versionNumber").and_then(|s| s.as_str()).unwrap_or("<unknown>"));
    }
    Ok(())
}

pub async fn list_ideas(limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/ideas?limit={}", cfg.api_base_url, limit);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("アイデア取得に失敗しました: {}", resp.status()));
    }
    let paginated: PaginatedResponse<ApiIdea> = resp.json().await?;
    for idea in paginated.data {
        println!("- {}: {}", idea.id, idea.title);
    }
    Ok(())
}

pub async fn list_comments(slug: &str) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/comments", cfg.api_base_url, slug);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("コメント取得に失敗しました: {}", resp.status()));
    }
    let paginated: PaginatedResponse<ApiComment> = resp.json().await?;
    for c in paginated.data {
        let author = c.author.map_or("匿名".to_string(), |a| a.username);
        println!("[{}] {}: {}", c.id, author, c.content);
    }
    Ok(())
}

pub async fn post_comment(slug: &str, content: &str) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/comments", cfg.api_base_url, slug);
    #[derive(Serialize)]
    struct Body<'a> {
        content: &'a str,
    }
    let resp = client.post(&url).json(&Body { content }).send().await?;
    if resp.status().is_success() {
        println!("✅ コメントを投稿しました。");
        Ok(())
    } else {
        Err(anyhow!("コメント投稿に失敗しました: {}", resp.status()))
    }
}

pub async fn sync_project(slug: &str) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/projects/{}/sync", cfg.api_base_url, slug);
    let resp = client.post(&url).send().await?;
    if resp.status().is_success() {
        println!("✅ プロジェクトを同期しました。");
        Ok(())
    } else {
        Err(anyhow!("同期に失敗しました: {}", resp.status()))
    }
}
