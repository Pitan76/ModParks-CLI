// src/commands/login.rs
use anyhow::{Result, bail};
use std::io::{self, Write};
use crate::config::Config;
use crate::api_client;

pub async fn login(api_key: &str) -> Result<()> {
    let mut cfg = Config::load()?;
    cfg.api_key = api_key.to_string();
    cfg.save()?;
    println!("API キーを保存しました。");
    Ok(())
}

pub async fn interactive_login() -> Result<()> {
    let mut cfg = Config::load()?;
    
    print!("ユーザーIDまたはメールアドレス: ");
    io::stdout().flush()?;
    let mut identifier = String::new();
    io::stdin().read_line(&mut identifier)?;
    let identifier = identifier.trim();
    
    let password = rpassword::prompt_password("パスワード: ")?;
    
    // First attempt without TOTP
    let resp = api_client::auth_login(&cfg, identifier, &password, None).await?;
    
    if let Some(err) = resp.error {
        bail!("ログイン失敗: {}", err);
    }
    
    let api_key = if resp.requires_2fa == Some(true) {
        print!("TOTPコード (2段階認証): ");
        io::stdout().flush()?;
        let mut totp = String::new();
        io::stdin().read_line(&mut totp)?;
        let totp = totp.trim();
        
        let resp_totp = api_client::auth_login(&cfg, identifier, &password, Some(totp)).await?;
        if let Some(err) = resp_totp.error {
            bail!("2FA認証失敗: {}", err);
        }
        
        resp_totp.api_key.ok_or_else(|| anyhow::anyhow!("APIキーが取得できませんでした"))?
    } else {
        resp.api_key.ok_or_else(|| anyhow::anyhow!("APIキーが取得できませんでした"))?
    };
    
    cfg.api_key = api_key;
    cfg.save()?;
    println!("ログイン成功！API キーを保存しました。");
    
    Ok(())
}
