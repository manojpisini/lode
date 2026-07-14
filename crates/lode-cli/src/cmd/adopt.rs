#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{
    analyze_project, format_adoption_report,
    load_project_config,
    ProjectConfig, ProjectSection, SCHEMA_VERSION,
    LodeError, register_project,
};

use crate::OutputFormat;

pub(crate) fn adopt_command(
    path: Option<Utf8PathBuf>,
    apply: bool,
    dry_run: bool,
    output: OutputFormat,
) -> lode_core::Result<()> {
    let dir = path.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| Utf8PathBuf::try_from(p).ok())
            .unwrap_or_else(|| Utf8PathBuf::from("."))
    });

    let existing_config = if dir.join(".lode").join("project.toml").exists() {
        load_project_config(&dir).ok()
    } else {
        None
    };

    if existing_config.is_some() && !apply {
        if output.should_use_json() {
            println!("{{\"error\": \"project already has a LODE manifest\"}}");
        } else {
            println!("this project already has a LODE manifest");
            println!("  use --apply to overwrite with new analysis");
        }
        return Ok(());
    }

    let report = analyze_project(&dir);

    if apply {
        if dry_run {
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|e| LodeError::Message(e.to_string()))?
                );
            } else {
                println!("[dry-run] would apply adoption plan");
                println!("  profile: {}", report.recommended_profile);
                if !report.recommended_components.is_empty() {
                    println!("  components: {}", report.recommended_components.join(", "));
                }
                println!("  languages:");
                for lang in &report.languages {
                    println!("    - {}", lang.name);
                }
            }
            return Ok(());
        }

        let profile = &report.recommended_profile;
        let components = &report.recommended_components;
        let primary_lang = report.languages.first().map(|l| l.name.as_str());

        let config = ProjectConfig {
            schema_version: SCHEMA_VERSION,
            project: ProjectSection {
                name: report.project_name.clone(),
                created_by: "lode".to_string(),
                created_at: crate::now_timestamp(),
                profile: profile.clone(),
                components: components.clone(),
                language: primary_lang.map(|s| s.to_string()),
                toolchain: if report.toolchains.is_empty() {
                    None
                } else {
                    Some(report.toolchains.clone())
                },
                assets: if report.detected_assets.is_empty() {
                    None
                } else {
                    Some(report.detected_assets.clone())
                },
                dependencies: None,
            },
        };

        let lode_dir = dir.join(".lode");
        fs::create_dir_all(lode_dir.as_std_path()).map_err(|e| LodeError::Io {
            path: lode_dir.as_str().into(),
            source: e,
        })?;

        let config_path = dir.join(".lode").join("project.toml");
        let raw = toml::to_string_pretty(&config)
            .map_err(|e| LodeError::Message(e.to_string()))?;
        fs::write(config_path.as_std_path(), &raw).map_err(|e| LodeError::Io {
            path: config_path.as_str().into(),
            source: e,
        })?;

        if output.should_use_json() {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .map_err(|e| LodeError::Message(e.to_string()))?
            );
        } else {
            println!("adopted {} with {} profile", report.project_name, profile);
            println!("  wrote: {}", config_path);
            if !components.is_empty() {
                println!("  components: {}", components.join(", "));
            }
        }

        let _ = register_project(&report.project_name, &dir, profile);
    } else {
        if output.should_use_json() {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .map_err(|e| LodeError::Message(e.to_string()))?
            );
        } else {
            println!("{}", format_adoption_report(&report));
        }
    }

    Ok(())
}
