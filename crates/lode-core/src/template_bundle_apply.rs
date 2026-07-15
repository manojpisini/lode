use std::collections::HashMap;
use std::path::Path;

use crate::error::{LodeError, Result};
use crate::template::RenderContext;
use crate::template_bundle::*;

/// Result of applying a template bundle
#[derive(Debug, Clone, Default)]
pub struct ApplyReport {
    pub files_written: Vec<String>,
    pub files_skipped: Vec<String>,
    pub assets_copied: Vec<String>,
    pub assets_skipped: Vec<String>,
    pub directories_created: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Apply a template bundle directory to a target directory
pub fn apply_bundle(
    bundle_dir: &Path,
    target_dir: &Path,
    values: &HashMap<String, String>,
    on_conflict: &str,
    dry_run: bool,
) -> Result<ApplyReport> {
    let manifest = load_template_bundle(bundle_dir)?;
    apply_manifest(
        &manifest,
        bundle_dir,
        target_dir,
        values,
        on_conflict,
        dry_run,
    )
}

/// Apply a loaded manifest to a target directory
pub fn apply_manifest(
    manifest: &TemplateManifest,
    bundle_dir: &Path,
    target_dir: &Path,
    values: &HashMap<String, String>,
    on_conflict: &str,
    dry_run: bool,
) -> Result<ApplyReport> {
    let mut report = ApplyReport::default();

    // 1. Build render context from variables + values
    let context = build_context(manifest, values, &mut report);

    // 2. Create directories
    for dir in &manifest.directories {
        let dir_path = resolve_path(&dir.path, &context, &mut report);
        let full_path = target_dir.join(&dir_path);
        if dry_run {
            report.directories_created.push(dir_path);
        } else if let Err(e) = create_dir_if_needed(&full_path, dir.mode.as_deref()) {
            report.errors.push(format!("directory {}: {e}", dir.path));
        } else {
            report.directories_created.push(dir_path);
            // handle .gitkeep and .gitignore
            if dir.keep {
                if !full_path.join(".gitkeep").exists() {
                    let _ = std::fs::write(full_path.join(".gitkeep"), "");
                }
            }
            if dir.gitignore_contents {
                let gi = full_path.join(".gitignore");
                if !gi.exists() {
                    let _ = std::fs::write(gi, "*\n");
                }
            }
        }
    }

    // 3. Infer parent dirs from file/asset destinations
    for f in &manifest.files {
        let dest = resolve_path(&f.path, &context, &mut report);
        if !dest.is_empty() {
            if let Some(parent) = Path::new(&dest).parent() {
                if !parent.as_os_str().is_empty() {
                    let full = target_dir.join(parent);
                    if !full.exists() && !dry_run {
                        let _ = create_dir_if_needed(&full, None);
                    }
                }
            }
        }
    }
    for a in &manifest.assets {
        let dest = if a.render_path {
            resolve_path(&a.destination, &context, &mut report)
        } else {
            a.destination.clone()
        };
        if !dest.is_empty() {
            if let Some(parent) = Path::new(&dest).parent() {
                if !parent.as_os_str().is_empty() {
                    let full = target_dir.join(parent);
                    if !full.exists() && !dry_run {
                        let _ = create_dir_if_needed(&full, None);
                    }
                }
            }
        }
    }

    // 4. Render and write inline files
    for f in &manifest.files {
        if let Some(cond) = &f.condition {
            if !eval_condition(cond, &context) {
                continue;
            }
        }
        let dest = resolve_path(&f.path, &context, &mut report);
        let full_path = target_dir.join(&dest);
        let rendered = if f.render {
            crate::template::render_template(&f.content, &context)
        } else {
            f.content.clone()
        };

        if full_path.exists() {
            let policy = f.overwrite.as_deref().unwrap_or(on_conflict);
            match policy {
                "skip" | "error" => {
                    report
                        .files_skipped
                        .push(format!("{} (policy: {})", dest, policy));
                    if policy == "error" {
                        report.errors.push(format!(
                            "file exists and overwrite policy is 'error': {dest}"
                        ));
                    }
                    continue;
                }
                _ => {}
            }
        }

        if !dry_run {
            if let Err(e) = std::fs::write(&full_path, &rendered) {
                report.errors.push(format!("write {}: {e}", dest));
                continue;
            }
        }
        report.files_written.push(dest);
    }

    // 5. Copy assets
    for a in &manifest.assets {
        if let Some(cond) = &a.condition {
            if !eval_condition(cond, &context) {
                continue;
            }
        }
        if !a.platforms.is_empty() {
            let current = std::env::consts::OS;
            if !a.platforms.iter().any(|p| p == current) {
                continue;
            }
        }
        if !a.architectures.is_empty() {
            let current = std::env::consts::ARCH;
            if !a.architectures.iter().any(|p| p == current) {
                continue;
            }
        }

        let dest = if a.render_path {
            resolve_path(&a.destination, &context, &mut report)
        } else {
            a.destination.clone()
        };
        let full_dest = target_dir.join(&dest);
        let source_path = bundle_dir.join(&a.source);

        if !source_path.exists() {
            if a.optional {
                report
                    .warnings
                    .push(format!("optional asset not found: {}", a.source));
                continue;
            }
            report.errors.push(format!(
                "asset not found: {} (looked at {})",
                a.source,
                source_path.display()
            ));
            continue;
        }

        if full_dest.exists() {
            let policy = a.overwrite.as_deref().unwrap_or(on_conflict);
            match policy {
                "skip" | "error" => {
                    report
                        .assets_skipped
                        .push(format!("{} (policy: {})", dest, policy));
                    if policy == "error" {
                        report.errors.push(format!("asset exists: {dest}"));
                    }
                    continue;
                }
                _ => {}
            }
        }

        if !dry_run {
            if let Err(e) = std::fs::copy(&source_path, &full_dest) {
                report.errors.push(format!("copy asset {}: {e}", dest));
                continue;
            }
            if a.executable {
                set_executable(&full_dest);
            }
        }
        report.assets_copied.push(dest);
    }

    Ok(report)
}

fn build_context(
    manifest: &TemplateManifest,
    values: &HashMap<String, String>,
    report: &mut ApplyReport,
) -> RenderContext {
    let mut ctx = RenderContext::new();

    // Apply defaults, then user values
    for v in &manifest.variables {
        let val = if let Some(user_val) = values.get(&v.name) {
            user_val.clone()
        } else if let Some(ref def) = v.default {
            def.as_str().unwrap_or("").to_string()
        } else {
            String::new()
        };

        if v.required && val.is_empty() && !values.contains_key(&v.name) {
            report.warnings.push(format!(
                "required variable '{}' not provided, using empty",
                v.name
            ));
        }

        ctx = ctx.with(&v.name, &val);
    }

    // Add derived variables
    if let Some(project) = values.get("project").or_else(|| values.get("name")) {
        ctx = ctx.with("project", project);
        ctx = ctx.with("ident", &crate::template::slug_to_ident(project));
        ctx = ctx.with("class", &crate::template::slug_to_class(project));
        ctx = ctx.with("camel", &slug_to_camel(project));
        ctx = ctx.with("constant", &slug_to_constant(project));
        ctx = ctx.with("title", &slug_to_title(project));
    }

    // Platform and arch
    ctx = ctx.with("platform", std::env::consts::OS);
    ctx = ctx.with("architecture", std::env::consts::ARCH);

    ctx
}

fn slug_to_camel(s: &str) -> String {
    let class = crate::template::slug_to_class(s);
    if class.is_empty() {
        return String::new();
    }
    let mut chars = class.chars();
    let first = chars.next().unwrap().to_lowercase().to_string();
    first + chars.as_str()
}

fn slug_to_constant(s: &str) -> String {
    crate::template::slug_to_ident(s).to_uppercase()
}

fn slug_to_title(s: &str) -> String {
    s.replace('-', " ")
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolve_path(template: &str, ctx: &RenderContext, _report: &mut ApplyReport) -> String {
    let rendered = crate::template::render_template(template, ctx);
    let stripped = rendered.trim_start_matches("./").trim_start_matches(".\\");
    if std::path::Path::new(&stripped)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return String::new();
    }
    stripped.to_string()
}

fn create_dir_if_needed(path: &Path, mode: Option<&str>) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|source| LodeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    // mode is informational on Windows; on Unix we'd use std::os::unix::fs::PermissionsExt
    let _ = mode;
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        let mode = perms.mode();
        perms.set_mode(mode | 0o111);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

/// Simple condition evaluator. Supports:
/// - `feature.X == true/false`
/// - `variable == 'value'`
/// - `platform == 'windows'`
/// - `architecture == 'x86_64'`
/// - `variable` (truthy check)
fn eval_condition(cond: &str, ctx: &RenderContext) -> bool {
    let trimmed = cond.trim();

    // Handle `feature.X == true/false`
    if let Some(val) = trimmed.strip_prefix("feature.") {
        let parts: Vec<&str> = val.splitn(2, "==").collect();
        if parts.len() == 2 {
            let feature = parts[0].trim();
            let expected = parts[1].trim().trim_matches('\'');
            let ctx_val = ctx.get(feature).unwrap_or("");
            // features default to false
            let expected_bool = expected == "true";
            let actual_bool = ctx_val == "true" || ctx_val == "1";
            return expected_bool == actual_bool;
        }
    }

    // Handle `platform == 'windows'` or `architecture == 'x86_64'`
    if let Some(eq_pos) = trimmed.find("==") {
        let key = trimmed[..eq_pos].trim();
        let val = trimmed[eq_pos + 2..].trim().trim_matches('\'').trim();
        let actual = ctx.get(key).unwrap_or("");
        return actual == val;
    }

    // Plain variable name: truthy check
    let ctx_val = ctx.get(trimmed).unwrap_or("");
    !ctx_val.is_empty() && ctx_val != "false" && ctx_val != "0"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context() -> RenderContext {
        RenderContext::new()
            .with("project", "my-app")
            .with("port", "8080")
            .with("platform", "windows")
            .with("architecture", "x86_64")
    }

    #[test]
    fn test_eval_condition_feature_true() {
        let ctx = RenderContext::new().with("auth", "true");
        assert!(eval_condition("feature.auth == true", &ctx));
    }

    #[test]
    fn test_eval_condition_feature_false() {
        let ctx = RenderContext::new();
        assert!(!eval_condition("feature.auth == true", &ctx));
    }

    #[test]
    fn test_eval_condition_platform() {
        let ctx = test_context();
        assert!(eval_condition("platform == 'windows'", &ctx));
        assert!(!eval_condition("platform == 'linux'", &ctx));
    }

    #[test]
    fn test_eval_condition_variable_eq() {
        let ctx = test_context();
        assert!(eval_condition("port == '8080'", &ctx));
        assert!(!eval_condition("port == '9090'", &ctx));
    }

    #[test]
    fn test_eval_condition_truthy() {
        let ctx = test_context();
        assert!(eval_condition("project", &ctx));
        assert!(eval_condition("port", &ctx));
    }

    #[test]
    fn test_slug_to_camel() {
        assert_eq!(slug_to_camel("my-app"), "myApp");
        assert_eq!(slug_to_camel("HelloWorld"), "helloWorld");
    }

    #[test]
    fn test_slug_to_constant() {
        assert_eq!(slug_to_constant("my-app"), "MY_APP");
    }

    #[test]
    fn test_slug_to_title() {
        assert_eq!(slug_to_title("my-app"), "My App");
    }

    #[test]
    fn test_resolve_path() {
        let ctx = test_context();
        let mut report = ApplyReport::default();
        assert_eq!(
            resolve_path("./src/{{ project }}/main.rs", &ctx, &mut report),
            "src/my-app/main.rs"
        );
    }

    #[test]
    fn test_apply_bundle_dry_run() {
        let dir = std::env::temp_dir().join(format!(
            "lode-apply-test-{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("out");
        let report = apply_manifest(
            &crate::template_bundle::sample_manifest(),
            &dir,
            &target,
            &[("project".into(), "test-app".into())]
                .into_iter()
                .collect(),
            "replace",
            true,
        )
        .unwrap();
        // all errors should be about missing asset sources
        assert!(report.errors.iter().all(|e| e.contains("asset not found")));
        assert!(!target.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
