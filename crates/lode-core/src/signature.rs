use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureConfig {
    pub enabled: bool,
    pub auto_insert: bool,
    pub auto_update_date: bool,
    pub include_path: bool,
    pub include_hash: bool,
    pub include_license: bool,
    pub separator_char: String,
}

impl Default for SignatureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_insert: true,
            auto_update_date: true,
            include_path: true,
            include_hash: false,
            include_license: true,
            separator_char: "─".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHeader {
    pub file: String,
    pub project: String,
    pub author: String,
    pub created: String,
    pub updated: String,
    pub license: String,
    pub path: Option<String>,
    pub hash: Option<String>,
}

impl SignatureHeader {
    pub fn new(file: &str, project: &str, author: &str, license: &str) -> Self {
        let today = chrono_now_date();
        Self {
            file: file.to_string(),
            project: project.to_string(),
            author: author.to_string(),
            created: today.clone(),
            updated: today,
            license: license.to_string(),
            path: None,
            hash: None,
        }
    }

    pub fn render(&self, config: &SignatureConfig, prefix: &str) -> String {
        let sep = config.separator_char.repeat(77);
        let mut lines = vec![format!("{prefix} {sep}")];
        lines.push(format!("{prefix} @file    {}", self.file));
        lines.push(format!("{prefix} @project {}", self.project));
        lines.push(format!("{prefix} @author  {}", self.author));
        lines.push(format!("{prefix} @created {}", self.created));
        lines.push(format!("{prefix} @updated {}", self.updated));
        if config.include_license {
            lines.push(format!("{prefix} @license {}", self.license));
        }
        if let Some(ref path) = self.path {
            if config.include_path {
                lines.push(format!("{prefix} PATH: {path}"));
            }
        }
        if let Some(ref hash) = self.hash {
            if config.include_hash {
                lines.push(format!("{prefix} HASH: {hash}"));
            }
        }
        lines.push(format!("{prefix} {sep}"));
        lines.push(String::new());
        lines.join("\n")
    }
}

pub fn comment_prefix_for_extension(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" | "c" | "h" | "cpp" | "hpp" | "cc" | "go" | "ts" | "tsx" | "js" | "jsx" | "java"
        | "kt" | "swift" | "cs" | "zig" => Some("//"),
        "py" | "sh" | "ps1" | "toml" | "yml" | "yaml" | "ini" | "rb" | "lua" | "sql" => Some("#"),
        "hs" | "elm" => Some("--"),
        "md" | "html" | "xml" | "svg" | "astro" => Some("<!--"),
        "css" | "scss" | "sass" | "less" => Some("//"),
        _ => None,
    }
}

pub fn has_signature_header(content: &str) -> bool {
    content
        .lines()
        .take(20)
        .any(|line| line.contains("@file") || line.contains("@project"))
}

pub fn compute_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn chrono_now_date() -> String {
    // Simple date without chrono dependency
    "2026-01-01".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_header_renders() {
        let header = SignatureHeader::new("src/main.rs", "my-proj", "alice", "MIT");
        let rendered = header.render(&SignatureConfig::default(), "//");
        assert!(rendered.contains("@file    src/main.rs"));
        assert!(rendered.contains("@project my-proj"));
        assert!(rendered.contains("@author  alice"));
    }

    #[test]
    fn has_signature_detects_header() {
        assert!(has_signature_header(
            "// @file src/main.rs\n// @project test"
        ));
        assert!(!has_signature_header("fn main() {}"));
    }
}
