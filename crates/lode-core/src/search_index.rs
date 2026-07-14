use std::collections::HashMap;
use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{build_catalog, config::LodeConfig, LodeError, Result};

const INDEX_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub schema_version: u32,
    pub indexed_at: String,
    pub total_entries: usize,
    pub word_index: HashMap<String, Vec<String>>,
    pub metadata_index: HashMap<String, Vec<IndexedEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEntry {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub status: String,
    pub maturity: String,
    pub quality_score: Option<u32>,
    pub languages: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub min_quality: Option<u32>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub status: String,
    pub maturity: String,
    pub quality_score: Option<u32>,
    pub languages: Vec<String>,
    pub relevance: f64,
}

pub fn index_path() -> Result<Utf8PathBuf> {
    let base = crate::install::global_asset_dir("templates")?;
    Ok(base.join(".lode").join("search-index.json"))
}

pub fn build_search_index(config: &LodeConfig) -> Result<SearchIndex> {
    let catalog = build_catalog(config);
    let mut word_index: HashMap<String, Vec<String>> = HashMap::new();
    let mut metadata_index: HashMap<String, Vec<IndexedEntry>> = HashMap::new();

    for entry in &catalog.entries {
        let id = entry.id.clone();
        let meta = IndexedEntry {
            id: id.clone(),
            kind: entry.kind.clone(),
            summary: entry.summary.clone(),
            status: entry.status.clone(),
            maturity: entry.maturity.clone(),
            quality_score: entry.quality_score,
            languages: entry.languages.clone(),
            tags: entry.tags.clone(),
        };

        for word in tokenize(&entry.id) {
            word_index.entry(word).or_default().push(id.clone());
        }
        for word in tokenize(&entry.summary) {
            word_index.entry(word).or_default().push(id.clone());
        }
        for intent in &entry.intents {
            for word in tokenize(intent) {
                word_index.entry(word).or_default().push(id.clone());
            }
        }
        for tag in &entry.tags {
            word_index.entry(tag.clone()).or_default().push(id.clone());
        }
        for lang in &entry.languages {
            word_index
                .entry(format!("lang:{}", lang))
                .or_default()
                .push(id.clone());
        }

        metadata_index
            .entry("kind:".to_string() + &entry.kind)
            .or_default()
            .push(meta.clone());
        metadata_index
            .entry("status:".to_string() + &entry.status)
            .or_default()
            .push(meta);
    }

    let idx = SearchIndex {
        schema_version: INDEX_SCHEMA_VERSION,
        indexed_at: format!(
            "unix:{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ),
        total_entries: catalog.entries.len(),
        word_index,
        metadata_index,
    };

    Ok(idx)
}

pub fn save_search_index(index: &SearchIndex) -> Result<()> {
    let path = index_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent.as_std_path()).map_err(|source| LodeError::Io {
            path: parent.as_str().into(),
            source,
        })?;
    }
    let json =
        serde_json::to_string_pretty(index).map_err(|e| LodeError::Message(e.to_string()))?;
    fs::write(path.as_std_path(), &json).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(())
}

pub fn load_search_index() -> Result<SearchIndex> {
    let path = index_path()?;
    let raw = fs::read_to_string(path.as_std_path()).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let idx: SearchIndex =
        serde_json::from_str(&raw).map_err(|e| LodeError::Message(e.to_string()))?;
    if idx.schema_version != INDEX_SCHEMA_VERSION {
        return Err(LodeError::SchemaMismatch {
            expected: INDEX_SCHEMA_VERSION,
            found: idx.schema_version,
        });
    }
    Ok(idx)
}

pub fn search_index(query: &SearchQuery) -> Result<Vec<SearchResult>> {
    search_index_with(query, None)
}

pub fn search_index_with(
    query: &SearchQuery,
    idx: Option<&SearchIndex>,
) -> Result<Vec<SearchResult>> {
    let idx = match idx {
        Some(i) => i.clone(),
        None => load_search_index()?,
    };
    let query_lower = query.query.to_lowercase();
    let query_words: Vec<String> = tokenize(&query_lower);

    let mut match_ids: Vec<(f64, &IndexedEntry)> = Vec::new();

    for meta in idx.metadata_index.values().flatten() {
        if let Some(ref kind) = query.kind {
            if meta.kind != *kind {
                continue;
            }
        }
        if let Some(ref status) = query.status {
            if meta.status != *status {
                continue;
            }
        }
        if let Some(mq) = query.min_quality {
            if meta.quality_score.unwrap_or(0) < mq {
                continue;
            }
        }

        let mut score = 0.0;
        let lower_id = meta.id.to_lowercase();
        let lower_summary = meta.summary.to_lowercase();

        if lower_id.contains(&query_lower) {
            score += 5.0;
        }
        if lower_summary.contains(&query_lower) {
            score += 10.0;
        }

        for word in &query_words {
            if let Some(matched_ids) = idx.word_index.get(word) {
                if matched_ids.contains(&meta.id) {
                    score += 3.0;
                }
            }
            for lang in &meta.languages {
                if lang.contains(word) || word.contains(&lang.to_lowercase()) {
                    score += 2.0;
                }
            }
        }

        if score > 0.0 {
            match_ids.push((score, meta));
        }
    }

    match_ids.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    match_ids.truncate(query.limit.max(1).min(100));

    Ok(match_ids
        .into_iter()
        .map(|(relevance, meta)| SearchResult {
            id: meta.id.clone(),
            kind: meta.kind.clone(),
            summary: meta.summary.clone(),
            status: meta.status.clone(),
            maturity: meta.maturity.clone(),
            quality_score: meta.quality_score,
            languages: meta.languages.clone(),
            relevance,
        })
        .collect())
}

fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    lower
        .split_whitespace()
        .flat_map(|w| {
            let clean: String = w.chars().filter(|c| c.is_alphanumeric()).collect();
            if clean.len() > 1 {
                vec![clean.clone()]
            } else {
                vec![]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_splits_and_cleans() {
        let tokens = tokenize("hello-world API v3");
        assert!(tokens.contains(&"helloworld".to_string()));
        assert!(tokens.contains(&"api".to_string()));
    }

    #[test]
    fn build_index_round_trip() {
        let config = crate::default_config();
        let idx = build_search_index(&config).unwrap();
        assert_eq!(idx.schema_version, 1);
        assert!(idx.total_entries > 0);
        assert!(!idx.word_index.is_empty());
    }

    #[test]
    fn search_finds_by_word() {
        let config = crate::default_config();
        let idx = build_search_index(&config).unwrap();
        let results = search_index_with(
            &SearchQuery {
                query: "rust".to_string(),
                kind: None,
                status: None,
                min_quality: None,
                limit: 10,
            },
            Some(&idx),
        )
        .unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn search_filters_by_kind() {
        let config = crate::default_config();
        let idx = build_search_index(&config).unwrap();
        let results = search_index_with(
            &SearchQuery {
                query: "rust".to_string(),
                kind: Some("profile".to_string()),
                status: None,
                min_quality: None,
                limit: 10,
            },
            Some(&idx),
        )
        .unwrap();
        for r in &results {
            assert_eq!(r.kind, "profile");
        }
    }

    #[test]
    fn search_filters_by_status() {
        let config = crate::default_config();
        let idx = build_search_index(&config).unwrap();
        let results = search_index_with(
            &SearchQuery {
                query: "rust".to_string(),
                kind: None,
                status: Some("stable".to_string()),
                min_quality: None,
                limit: 10,
            },
            Some(&idx),
        )
        .unwrap();
        for r in &results {
            assert_eq!(r.status, "stable");
        }
    }
}
