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

/// Width of the signature separator line (77 chars fits within 80-col terminals with comment prefix).
const SEPARATOR_WIDTH: usize = 77;

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
        let sep = config.separator_char.repeat(SEPARATOR_WIDTH);
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
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    crate::util::hex_lower(hasher.finalize())
}

fn chrono_now_date() -> String {
    let now = std::time::SystemTime::now();
    let since_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = since_epoch.as_secs();
    let days = total_secs / 86400;

    let mut year: u64 = 1970;
    let mut remaining_days = days;
    loop {
        let days_in_year =
            if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
                366
            } else {
                365
            };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let days_in_months =
        if (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
    let mut month: u64 = 1;
    for &dim in days_in_months.iter() {
        if remaining_days < dim {
            break;
        }
        remaining_days -= dim;
        month += 1;
    }
    let day = remaining_days + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
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
