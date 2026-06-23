// src/commands/download.rs
use anyhow::{Result, anyhow, Context};
use crate::config::Config;
use crate::api_client::build_client;
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

/// Download a version of a project.
pub async fn download_version(
    slug: String,
    version_opt: Option<String>,
    out_opt: Option<String>,
) -> Result<()> {
    let cfg = Config::load()?;
    let client = build_client(&cfg.api_key)?;
    
    // 1. バージョン一覧を取得
    let url = format!("{}/projects/{}/versions?limit=50", cfg.api_base_url, slug);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("バージョン情報の取得に失敗しました: {}", resp.status()));
    }
    let json: Value = resp.json().await?;
    let data_array = json.get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("APIレスポンスの形式が不正です (data配列がありません)"))?;

    if data_array.is_empty() {
        return Err(anyhow!("プロジェクト '{}' にはダウンロード可能なバージョンがありません。", slug));
    }

    // 2. 指定されたバージョン、または最新バージョンを探す
    let target_version = if let Some(ver) = version_opt {
        data_array.iter()
            .find(|v| v.get("versionNumber").and_then(|vn| vn.as_str()) == Some(&ver))
            .ok_or_else(|| anyhow!("指定されたバージョン '{}' は見つかりませんでした。", ver))?
    } else {
        // 先頭を最新とする（API側で降順ソートされている前提）
        data_array.first().unwrap()
    };

    let version_id = target_version.get("id").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("バージョンIDが取得できませんでした。"))?;
    let file_name = target_version.get("fileName").and_then(|v| v.as_str())
        .unwrap_or("download.jar");
    let file_url_rel = target_version.get("fileUrl").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("ダウンロードURLが取得できませんでした。"))?;

    // fileUrl が相対パスの場合はベースURLと結合する
    let download_url = if file_url_rel.starts_with("http") {
        file_url_rel.to_string()
    } else {
        // cfg.api_base_url は "/api/v1" などで終わっている場合がある。
        // file_url_rel は "/api/download?versionId=..." などの可能性がある。
        let base_host = cfg.api_base_url.split("/api/").next().unwrap_or(&cfg.api_base_url);
        format!("{}{}", base_host, file_url_rel)
    };

    println!("バージョン情報を取得しました。");
    println!("バージョン: {}", target_version.get("versionNumber").and_then(|v| v.as_str()).unwrap_or("Unknown"));
    println!("ファイル名: {}", file_name);
    println!("ダウンロードURLにアクセスします...");

    // 3. ダウンロードを実行
    let mut resp = client.get(&download_url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("ファイルのダウンロードに失敗しました: {}", resp.status()));
    }

    // 4. 出力先の決定
    let mut out_path = PathBuf::from(out_opt.unwrap_or_else(|| file_name.to_string()));
    if out_path.is_dir() {
        out_path.push(file_name);
    }

    // 5. ファイルへ書き込み
    let mut file = tokio::fs::File::create(&out_path).await
        .context(format!("出力先ファイルの作成に失敗しました: {:?}", out_path))?;
    
    let mut downloaded: u64 = 0;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
    }
    
    println!("ダウンロード完了: {:?} ({} bytes)", out_path, downloaded);
    Ok(())
}
