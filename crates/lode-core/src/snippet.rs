use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    error::{LodeError, Result},
    ValidatedRoot,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetConfig {
    pub auto_export: bool,
    pub export_format: String,
    pub export_path: PathBuf,
}

impl Default for SnippetConfig {
    fn default() -> Self {
        Self {
            auto_export: false,
            export_format: "json".to_string(),
            export_path: PathBuf::from(".snippets"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub lang: String,
    pub name: String,
    pub body: String,
    pub trigger: Option<String>,
    pub description: Option<String>,
}

impl Snippet {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|source| LodeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();
        let lang = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("txt")
            .to_string();
        let trigger = content
            .lines()
            .find(|l| l.starts_with("// trigger:") || l.starts_with("# trigger:"))
            .map(|l| {
                l.trim_start_matches("// trigger:")
                    .trim_start_matches("# trigger:")
                    .trim()
                    .to_string()
            });
        let description = content
            .lines()
            .find(|l| l.starts_with("// desc:") || l.starts_with("# desc:"))
            .map(|l| {
                l.trim_start_matches("// desc:")
                    .trim_start_matches("# desc:")
                    .trim()
                    .to_string()
            });
        Ok(Self {
            lang,
            name,
            body: content,
            trigger,
            description,
        })
    }
}

pub fn list_snippets(root: &Path) -> Result<Vec<Snippet>> {
    let snippets_dir = root.join(".snippets");
    if !snippets_dir.exists() {
        return Ok(Vec::new());
    }
    let mut snippets = Vec::new();
    visit_snippets(&snippets_dir, &mut snippets)?;
    Ok(snippets)
}

fn visit_snippets(dir: &Path, out: &mut Vec<Snippet>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            visit_snippets(&path, out)?;
        } else if let Ok(snippet) = Snippet::from_file(&path) {
            out.push(snippet);
        }
    }
    Ok(())
}

pub fn search_snippets<'a>(snippets: &'a [Snippet], query: &str) -> Vec<&'a Snippet> {
    let q = query.to_lowercase();
    snippets
        .iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&q)
                || s.body.to_lowercase().contains(&q)
                || s.description
                    .as_deref()
                    .map(|d| d.to_lowercase().contains(&q))
                    .unwrap_or(false)
                || s.trigger
                    .as_deref()
                    .map(|t| t.to_lowercase().contains(&q))
                    .unwrap_or(false)
        })
        .collect()
}

pub fn insert_snippet(snippet: &Snippet, target: &Path, line: usize) -> Result<()> {
    let mut content = fs::read_to_string(target).map_err(|source| LodeError::Io {
        path: target.to_path_buf(),
        source,
    })?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let insert_at = line.min(lines.len());
    for snippet_line in snippet.body.lines() {
        lines.insert(insert_at, snippet_line.to_string());
    }
    lines.push(String::new());
    content = lines.join("\n");
    let parent = target
        .parent()
        .ok_or_else(|| LodeError::Message(format!("missing parent: {}", target.display())))?;
    let name = target
        .file_name()
        .ok_or_else(|| LodeError::Message(format!("missing file name: {}", target.display())))?;
    ValidatedRoot::new(parent)?.write_atomic(Path::new(name), content)?;
    Ok(())
}

pub fn export_snippets(snippets: &[Snippet], format: &str) -> Result<String> {
    match format {
        "json" => {
            serde_json::to_string_pretty(snippets).map_err(|e| LodeError::Message(e.to_string()))
        }
        "toml" => toml::to_string_pretty(snippets).map_err(LodeError::from),
        _ => Err(LodeError::Message(format!(
            "unsupported export format: {format}"
        ))),
    }
}

pub fn add_snippet(root: &Path, snippet: &Snippet) -> Result<PathBuf> {
    let dir = Path::new(".snippets").join(&snippet.lang);
    let file_path = root
        .join(&dir)
        .join(format!("{}.{}", snippet.name, snippet.lang));
    let root = ValidatedRoot::new(root)?;
    root.create_dir_all(&dir)?;
    root.write_atomic(
        dir.join(format!("{}.{}", snippet.name, snippet.lang)),
        &snippet.body,
    )?;
    Ok(file_path)
}
pub fn remove_snippet(root: &Path, lang: &str, name: &str) -> Result<PathBuf> {
    let file_path = root
        .join(".snippets")
        .join(lang)
        .join(format!("{name}.{lang}"));
    if !file_path.exists() {
        return Err(LodeError::Message(format!(
            "snippet not found: {lang}/{name}"
        )));
    }
    ValidatedRoot::new(root)?.remove_file(
        Path::new(".snippets")
            .join(lang)
            .join(format!("{name}.{lang}")),
    )?;
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_remove_snippet_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let snippet = Snippet {
            lang: "rs".into(),
            name: "test_snip".into(),
            body: "fn hello() {}".into(),
            trigger: Some("hel".into()),
            description: None,
        };
        let path = add_snippet(temp.path(), &snippet).unwrap();
        assert!(path.exists());

        let removed = remove_snippet(temp.path(), "rs", "test_snip").unwrap();
        assert_eq!(removed, path);
        assert!(!path.exists());
    }

    #[test]
    fn search_finds_by_body() {
        let snippets = vec![
            Snippet {
                lang: "rs".into(),
                name: "alpha".into(),
                body: "let x = 42;".into(),
                trigger: None,
                description: None,
            },
            Snippet {
                lang: "py".into(),
                name: "beta".into(),
                body: "print('hello')".into(),
                trigger: None,
                description: None,
            },
        ];
        let results = search_snippets(&snippets, "print");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "beta");
    }
}
