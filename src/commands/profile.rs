// src/commands/profile.rs
use anyhow::{Result, anyhow};
use crate::config::Config;
use crate::api_client::{build_client, auth_me};
use serde_json::Value;

/// Get and display the current user's profile.
pub async fn display_profile() -> Result<()> {
    let cfg = Config::load()?;
    if cfg.api_key.is_empty() {
        return Err(anyhow!("ログインしていません。まず login コマンドを実行してください。"));
    }

    // まず /auth/me でユーザー名を取得
    let me = auth_me(&cfg).await?;
    let username = me.username;

    // 次に /users/[username] で詳細プロフィールを取得
    let client = build_client(&cfg.api_key)?;
    let url = format!("{}/users/{}", cfg.api_base_url, username);
    let resp = client.get(&url).send().await?;
    
    if !resp.status().is_success() {
        return Err(anyhow!("プロフィールの取得に失敗しました: {}", resp.status()));
    }
    
    let json: Value = resp.json().await?;
    if let Some(data) = json.get("data") {
        println!("==== Profile ====");
        println!("Username: {}", data.get("username").and_then(|v| v.as_str()).unwrap_or(""));
        println!("Display Name: {}", data.get("displayName").and_then(|v| v.as_str()).unwrap_or("(Not set)"));
        println!("Bio: {}", data.get("bio").and_then(|v| v.as_str()).unwrap_or("(Not set)"));
        println!("GitHub: {}", data.get("githubUsername").and_then(|v| v.as_str()).unwrap_or("(Not set)"));
        println!("Avatar URL: {}", data.get("avatarUrl").and_then(|v| v.as_str()).unwrap_or("(Not set)"));
        println!("Role: {}", me.role);
        println!("=================");
    } else {
        println!("プロフィールが見つかりませんでした。");
    }

    Ok(())
}
