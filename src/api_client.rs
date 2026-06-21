// src/api_client.rs
use anyhow::{Result, Context, anyhow};
use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT, AUTHORIZATION}};
use serde::{Serialize, Deserialize};
use serde_json::Value;
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

#[derive(Serialize)]
struct LoginPayload<'a> {
    identifier: &'a str,
    password: &'a str,
    #[serde(rename = "totpCode", skip_serializing_if = "Option::is_none")]
    totp_code: Option<&'a str>,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "apiKey")]
    pub api_key: Option<String>,
    #[serde(rename = "requires_2fa")]
    pub requires_2fa: Option<bool>,
    pub error: Option<String>,
}

/// IDとパスワード（および必要ならTOTP）を用いてログインし、API キーまたは2FA要求を返す。
pub async fn auth_login(cfg: &Config, identifier: &str, password: &str, totp_code: Option<&str>) -> Result<LoginResponse> {
    let client = build_client("")?;
    let url = format!("{}/auth/login", cfg.api_base_url);
    let payload = LoginPayload {
        identifier,
        password,
        totp_code,
    };

    let resp = client.post(&url).json(&payload).send().await?;
    let status = resp.status();
    
    let data: Value = resp.json().await?;
    
    if status.is_success() {
        if let Some(key) = data.get("apiKey").and_then(|v| v.as_str()) {
            return Ok(LoginResponse {
                api_key: Some(key.to_string()),
                requires_2fa: None,
                error: None,
            });
        }
    } else if status == 401 {
        if let Some(req_2fa) = data.get("requires_2fa").and_then(|v| v.as_bool()) {
            return Ok(LoginResponse {
                api_key: None,
                requires_2fa: Some(req_2fa),
                error: None,
            });
        } else if let Some(err) = data.get("error").and_then(|v| v.as_str()) {
            return Ok(LoginResponse {
                api_key: None,
                requires_2fa: None,
                error: Some(err.to_string()),
            });
        }
    }

    let msg = data.get("error").and_then(|v| v.as_str()).unwrap_or("不明なエラー");
    Err(anyhow!("ログイン失敗: {} ({})", msg, status))
}