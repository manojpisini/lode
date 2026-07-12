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
        && ["api_key", "apikey", "secret", "token", "password", "private_key"]
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
    if line.contains("ghp_") || line.contains("github_pat_") {
        return line
            .replace("ghp_", "ghp_[REDACTED]")
            .replace("github_pat_", "github_pat_[REDACTED]");
    }
    if (line.contains("AKIA")
        || line.contains("ASIA")
        || line.contains("ABIA")
        || line.contains("ACCA")
        || line.contains("AROA"))
        && line.len() >= 20
    {
        let mut result = String::with_capacity(line.len());
        let mut remaining = line;
        let prefixes = &["AKIA", "ASIA", "ABIA", "ACCA", "AROA"];
        while !remaining.is_empty() {
            let mut matched = false;
            for prefix in prefixes {
                if let Some(pos) = remaining.find(prefix) {
                    result.push_str(&remaining[..pos]);
                    result.push_str("[REDACTED]");
                    remaining = &remaining[pos + prefix.len()..];
                    matched = true;
                    break;
                }
            }
            if !matched {
                result.push_str(remaining);
                break;
            }
        }
        return result;
    }
    if line.contains("-----BEGIN") && line.contains("PRIVATE KEY-----") {
        return "[REDACTED PRIVATE KEY]".to_string();
    }
    line.to_string()
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
}
