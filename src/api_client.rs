// src/api_client.rs
use anyhow::{Result, Context};
use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT, AUTHORIZATION}};
use crate::config::Config;

/// Build a reqwest client with optional API key authentication.
///
/// * `api_key` – The API key from the config. If empty, the client is built without an Authorization header.
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
        .context("Failed to build reqwest client")?;
    Ok(client)
}
