use crate::error::Result;
use crate::models::CleanupItem;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CACHE_VERSION: u8 = 1;
const CACHE_FILE: &str = "scan_cache.json";

#[derive(Debug, Serialize, Deserialize)]
struct CachePayload {
    version: u8,
    created_at: i64,
    items: Vec<CleanupItem>,
}

pub fn load_cached_items() -> Result<Option<Vec<CleanupItem>>> {
    let path = cache_path();
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)?;
    let payload: CachePayload = serde_json::from_str(&content)?;
    if payload.version != CACHE_VERSION {
        return Ok(None);
    }

    Ok(Some(payload.items))
}

pub fn save_cached_items(items: &[CleanupItem]) -> Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = CachePayload {
        version: CACHE_VERSION,
        created_at: Utc::now().timestamp(),
        items: items.to_vec(),
    };
    let content = serde_json::to_string_pretty(&payload)?;
    fs::write(path, content)?;
    Ok(())
}

fn cache_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CACHE_HOME") {
        return Path::new(&dir).join("rcleaner").join(CACHE_FILE);
    }
    if let Ok(home) = std::env::var("HOME") {
        return Path::new(&home)
            .join(".cache")
            .join("rcleaner")
            .join(CACHE_FILE);
    }
    PathBuf::from(CACHE_FILE)
}
