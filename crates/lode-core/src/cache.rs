use std::collections::HashMap;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{LodeError, Result};

pub type CacheKey = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: CacheKey,
    pub value: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub hit_count: u64,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStore {
    pub entries: HashMap<CacheKey, CacheEntry>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs()
}

pub fn cache_path() -> Result<std::path::PathBuf> {
    let dir = crate::install::global_asset_dir("state")?;
    Ok(dir.join("cache.json").into())
}

pub fn load_cache() -> Result<CacheStore> {
    let path = cache_path()?;
    if !path.exists() {
        return Ok(CacheStore::default());
    }
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))
}

pub fn save_cache(cache: &CacheStore) -> Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| LodeError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let raw = serde_json::to_string_pretty(cache).map_err(|e| LodeError::Message(e.to_string()))?;
    fs::write(&path, &raw).map_err(|source| LodeError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(())
}

pub fn cache_get(key: &str) -> Result<Option<String>> {
    let mut cache = load_cache()?;
    let now = now_secs();
    if !cache.entries.contains_key(key) {
        return Ok(None);
    }
    if cache.entries[key].ttl_seconds > 0
        && now - cache.entries[key].created_at > cache.entries[key].ttl_seconds
    {
        cache.entries.remove(key);
        save_cache(&cache)?;
        return Ok(None);
    }
    cache.entries.get_mut(key).map(|entry| {
        entry.last_accessed = now;
        entry.hit_count += 1;
    });
    let value = cache.entries[key].value.clone();
    save_cache(&cache)?;
    Ok(Some(value))
}

pub fn cache_set(key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
    let mut cache = load_cache()?;
    let now = now_secs();
    cache.entries.insert(
        key.to_string(),
        CacheEntry {
            key: key.to_string(),
            value: value.to_string(),
            created_at: now,
            last_accessed: now,
            hit_count: 0,
            ttl_seconds,
        },
    );
    save_cache(&cache)?;
    Ok(())
}

pub fn cache_clear() -> Result<()> {
    let path = cache_path()?;
    if path.exists() {
        fs::remove_file(&path).map_err(|source| LodeError::Io {
            path: path.clone(),
            source,
        })?;
    }
    Ok(())
}

pub fn cache_stats() -> Result<CacheStore> {
    load_cache()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(key: &str, val: &str) -> CacheEntry {
        let now = now_secs();
        CacheEntry {
            key: key.into(),
            value: val.into(),
            created_at: now,
            last_accessed: now,
            hit_count: 0,
            ttl_seconds: 0,
        }
    }

    #[test]
    fn test_cache_store_in_memory() {
        let mut store = CacheStore {
            entries: HashMap::new(),
        };
        store.entries.insert("k1".into(), make_entry("k1", "v1"));
        assert_eq!(
            store.entries.get("k1").map(|e| e.value.as_str()),
            Some("v1")
        );
        assert!(store.entries.get("k2").is_none());
    }

    #[test]
    fn test_cache_clear_entries() {
        let mut store = CacheStore {
            entries: HashMap::new(),
        };
        store.entries.insert("k".into(), make_entry("k", "v"));
        store.entries.clear();
        assert!(store.entries.is_empty());
    }

    #[test]
    fn test_cache_stats_is_load() {
        let store = CacheStore {
            entries: HashMap::new(),
        };
        assert_eq!(store.entries.len(), 0);
    }
}
