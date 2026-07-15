use crate::secrets::SecretFinding;

pub fn redact(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let lines: Vec<&str> = input.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&redact_line(line));
    }
    result
}

fn redact_line(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let has_assignment = line.contains('=') || line.contains(':');
    if has_assignment
        && [
            "api_key",
            "apikey",
            "secret",
            "token",
            "password",
            "private_key",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        && !lower.contains("changeme")
        && !lower.contains("example")
    {
        return if let Some(eq_pos) = line.find('=') {
            format!("{} [REDACTED]", &line[..eq_pos + 1])
        } else if let Some(col_pos) = line.find(':') {
            format!("{} [REDACTED]", &line[..col_pos + 1])
        } else {
            line.to_string()
        };
    }
    if let Some(redacted) = redact_github_token(line) {
        return redacted;
    }
    if let Some(redacted) = redact_aws_key(line) {
        return redacted;
    }
    if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----") {
        return "[REDACTED PRIVATE KEY]".to_string();
    }
    if let Some(redacted) = redact_high_entropy(line) {
        return redacted;
    }
    line.to_string()
}

fn redact_github_token(line: &str) -> Option<String> {
    for (prefix, prefix_len) in &[("ghp_", 4usize), ("github_pat_", 11)] {
        if let Some(pos) = line.find(prefix) {
            let rest = &line[pos + prefix_len..];
            let token_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if token_end >= 4 {
                if let Some(eq) = line[..pos].rfind('=') {
                    return Some(format!("{}=[REDACTED]", &line[..eq]));
                }
                return Some(format!("{}[REDACTED]{}", &line[..pos], &rest[token_end..]));
            }
        }
    }
    None
}

fn redact_aws_key(line: &str) -> Option<String> {
    if line.len() < 20 {
        return None;
    }
    let prefixes = &["AKIA", "ASIA", "ABIA", "ACCA", "AROA"];
    for prefix in prefixes {
        if let Some(pos) = line.find(prefix) {
            let rest = &line[pos..];
            let token_end = rest
                .find(|c: char| !c.is_ascii_alphanumeric())
                .unwrap_or(rest.len());
            if token_end >= 20 {
                return Some(format!("{}[REDACTED]{}", &line[..pos], &rest[token_end..]));
            }
        }
    }
    None
}

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let len = s.len() as f64;
    let mut freq = [0u32; 256];
    for &b in s.as_bytes() {
        freq[b as usize] += 1;
    }
    let mut entropy = 0.0;
    for &count in freq.iter() {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn redact_high_entropy(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
            {
                i += 1;
            }
            let token = &line[start..i];
            if token.len() >= 16 && shannon_entropy(token) > 4.5 {
                return Some(format!("{}[REDACTED]{}", &line[..start], &line[i..]));
            }
        } else {
            i += 1;
        }
    }
    None
}

pub fn redact_findings(input: &str, findings: &[SecretFinding]) -> String {
    if findings.is_empty() {
        return input.to_string();
    }
    let mut result = String::with_capacity(input.len());
    let lines: Vec<&str> = input.lines().collect();
    let finding_lines: std::collections::BTreeSet<usize> =
        findings.iter().map(|f| f.line).collect();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let line_num = i + 1;
        if finding_lines.contains(&line_num) {
            result.push_str(&redact_line(line));
        } else {
            result.push_str(line);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::SecretFinding;
    use camino::Utf8PathBuf;

    #[test]
    fn redacts_suspicious_assignment() {
        let result = redact("API_KEY=super-secret-value");
        assert_eq!(result, "API_KEY= [REDACTED]");
    }

    #[test]
    fn redacts_github_token() {
        let result = redact("token=ghp_abc123def456");
        assert!(result.contains("[REDACTED]"));
        assert!(!result.contains("ghp_abc123"));
    }

    #[test]
    fn redacts_github_pat() {
        let result = redact("GITHUB_PAT=github_pat_abc123");
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_aws_key() {
        let result = redact("AWS_ACCESS_KEY=AKIA1234567890ABCDEF");
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_private_key_line() {
        let result = redact("-----BEGIN PRIVATE KEY-----");
        assert_eq!(result, "[REDACTED PRIVATE KEY]");
    }

    #[test]
    fn skips_changeme() {
        let result = redact("API_KEY=changeme");
        assert_eq!(result, "API_KEY=changeme");
    }

    #[test]
    fn skips_example() {
        let result = redact("API_KEY=example-key");
        assert_eq!(result, "API_KEY=example-key");
    }

    #[test]
    fn empty_input() {
        assert_eq!(redact(""), "");
    }

    #[test]
    fn single_line_no_secret() {
        assert_eq!(redact("fn main() {}"), "fn main() {}");
    }

    #[test]
    fn multiline_with_mixed() {
        let input = "fn main() {\nlet api_key = \"secret\";\nprintln!(\"ok\");\n}";
        let result = redact(input);
        assert!(result.contains("[REDACTED]"));
        assert!(result.contains("fn main() {"));
        assert!(result.contains("println!"));
        assert!(result.contains("}"));
    }

    #[test]
    fn redact_findings_only_targets_finding_lines() {
        let input = "safe line\nAPI_KEY=secret\nanother safe line";
        let findings = vec![SecretFinding {
            path: Utf8PathBuf::from("test"),
            line: 2,
            kind: "suspicious credential assignment".to_string(),
        }];
        let result = redact_findings(input, &findings);
        assert_eq!(result, "safe line\nAPI_KEY= [REDACTED]\nanother safe line");
    }

    #[test]
    fn redact_findings_noop_for_empty_findings() {
        let input = "safe line";
        let result = redact_findings(input, &[]);
        assert_eq!(result, "safe line");
    }

    #[test]
    fn redacts_high_entropy_standalone_token() {
        let token = "aB3dE5fG7hI9jK1lM2nO4pQ6rS8tU0vW1xY3zZ5";
        let line = format!("some_prefix_{token}_suffix");
        let result = redact(&line);
        assert!(
            result.contains("[REDACTED]"),
            "high-entropy token should be redacted"
        );
        assert!(!result.contains(&token), "token value should not appear");
    }

    #[test]
    fn skips_low_entropy_long_token() {
        let line = "src/main/aaaaaaaaaaaaaaaa/";
        let result = redact(line);
        assert_eq!(result, "src/main/aaaaaaaaaaaaaaaa/");
    }

    #[test]
    fn skips_short_high_entropy() {
        let line = "aB3dE5fG"; // 8 chars, under 16 threshold
        let result = redact(line);
        assert_eq!(result, line);
    }
}
