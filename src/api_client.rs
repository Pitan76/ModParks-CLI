// src/api_client.rs
use anyhow::{Result, Context};
use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT, AUTHORIZATION}};
use serde::{Serialize, Deserialize};
use crate::config::Config;
use crate::cache;

/// reqwest クライアントを構築する。
pub fn build_client(api_key: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("modparks-cli/0.1.0 (github.com/pitan76/ModParks-CLI)"),
    );
    if !api_key.is_empty() {
        let value = format!("Bearer {}", api_key);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&value).context("Invalid API key header")?);
    }
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .context("reqwest クライアントの構築に失敗しました")?;
    Ok(client)
}

/// キャッシュ付き GET リクエスト。
/// cfg.cache_enabled が true かつ TTL 内ならキャッシュから返す。
pub async fn cached_get<T>(url: &str, cfg: &Config) -> Result<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    // キャッシュ有効かつヒットした場合はそのまま返す
    if cfg.cache_enabled {
        if let Some(cached) = cache::get::<T>(url, cfg.cache_ttl_seconds) {
            return Ok(cached);
        }
    }

    // HTTP リクエスト
    let client = build_client(&cfg.api_key)?;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("HTTPエラー: {}", resp.status()));
    }
    let data: T = resp.json().await?;

    // キャッシュに保存（失敗しても無視）
    if cfg.cache_enabled {
        let _ = cache::set(url, &data);
    }

    Ok(data)
}
