// src/commands/version.rs
use anyhow::{Result, anyhow, Context};
use crate::config::Config;
use crate::api_client::{build_client, cached_get};
use reqwest::multipart;

/// List versions of a project.
pub async fn list_versions(slug: &str, limit: u32) -> Result<()> {
    let cfg = Config::load()?;
    let url = format!("{}/projects/{}/versions?limit={}", cfg.api_base_url, slug, limit);
    let json: serde_json::Value = cached_get(&url, &cfg).await
        .map_err(|_| anyhow!("バージョン取得に失敗しました"))?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Create a new version for a project.
pub async fn create_version(
    slug: String,
    file: Option<String>,
    url_opt: Option<String>,
    file_name: Option<String>,
    version_number: String,
    loaders: Vec<String>,
    mc_versions: Vec<String>,
    changelog: Option<String>,
) -> Result<()> {
    if file.is_none() && url_opt.is_none() {
        return Err(anyhow!("--file または --url のいずれかを指定してください。"));
    }

    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    let endpoint = format!("{}/projects/{}/versions", cfg.api_base_url, slug);

    let mut form = multipart::Form::new()
        .text("versionNumber", version_number);

    for loader in loaders {
        form = form.text("loaders", loader);
    }
    for mc in mc_versions {
        form = form.text("mcVersions", mc);
    }
    if let Some(cl) = changelog {
        form = form.text("changelog", cl);
    }

    if let Some(file_path) = file {
        let path = std::path::Path::new(&file_path);
        if !path.exists() {
            return Err(anyhow!("指定されたファイルが見つかりません: {}", file_path));
        }
        let file_bytes = tokio::fs::read(&path).await.context("ファイルの読み込みに失敗しました")?;
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload.jar")
            .to_string();

        let part = multipart::Part::bytes(file_bytes)
            .file_name(name)
            .mime_str("application/octet-stream")?;
        form = form.part("file", part);
    } else if let Some(external_url) = url_opt {
        form = form.text("fileUrl", external_url);
        if let Some(name) = file_name {
            form = form.text("fileName", name);
        }
    }

    let resp = client.post(&endpoint)
        .multipart(form)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("バージョンの作成に失敗しました ({}): {}", status, body));
    }

    let json: serde_json::Value = resp.json().await?;
    println!("バージョンを作成しました:\n{}", serde_json::to_string_pretty(&json)?);

    Ok(())
}
