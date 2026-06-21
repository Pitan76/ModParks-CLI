// src/cache.rs
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use dirs::cache_dir;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    cached_at: u64,
}

fn cache_dir_path() -> Result<PathBuf> {
    let mut path = cache_dir().context("キャッシュディレクトリを取得できませんでした")?;
    path.push("modparks-cli");
    fs::create_dir_all(&path).context("キャッシュディレクトリの作成に失敗しました")?;
    Ok(path)
}

fn cache_key(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hex::encode(hasher.finalize())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// キャッシュからデータを読み込む。TTL 切れまたは無効なら None を返す。
pub fn get<T: for<'de> Deserialize<'de>>(url: &str, ttl_seconds: u64) -> Option<T> {
    let dir = cache_dir_path().ok()?;
    let path = dir.join(format!("{}.json", cache_key(url)));
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let entry: CacheEntry<T> = serde_json::from_str(&content).ok()?;
    let age = now_secs().saturating_sub(entry.cached_at);
    if age > ttl_seconds {
        return None;
    }
    Some(entry.data)
}

/// データをキャッシュに書き込む。
pub fn set<T: Serialize>(url: &str, data: &T) -> Result<()> {
    let dir = cache_dir_path()?;
    let path = dir.join(format!("{}.json", cache_key(url)));
    let entry = CacheEntry {
        data,
        cached_at: now_secs(),
    };
    let json = serde_json::to_string(&entry).context("キャッシュのシリアライズに失敗しました")?;
    fs::write(&path, json).context("キャッシュの書き込みに失敗しました")?;
    Ok(())
}

/// キャッシュを全クリアする。
pub fn clear() -> Result<usize> {
    let dir = cache_dir_path()?;
    let mut count = 0;
    for entry in fs::read_dir(&dir).context("キャッシュディレクトリを読み込めませんでした")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            fs::remove_file(&path)?;
            count += 1;
        }
    }
    Ok(count)
}