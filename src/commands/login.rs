// src/commands/login.rs
use anyhow::Result;
use crate::config::Config;

pub async fn login(api_key: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    cfg.api_key = api_key.to_string();
    cfg.save()?;
    println!("API キーを保存しました。");
    Ok(())
}
