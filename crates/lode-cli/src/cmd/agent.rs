#![deny(unsafe_code)]

use lode_core::{build_catalog, generate_agent_policy, resolve_intent, BootstrapInfo, LodeError};

use crate::output;
use crate::AgentCommand;
use crate::OutputFormat;

pub(crate) fn agent_command(command: AgentCommand) -> lode_core::Result<()> {
    match command {
        AgentCommand::Sync => crate::agent_sync(),
        AgentCommand::Status => crate::agent_status(),
        AgentCommand::Export { out } => crate::agent_export(out),
        AgentCommand::Plan { command } => crate::agent_plan(command),
        AgentCommand::Bootstrap { output } => agent_bootstrap(output),
        AgentCommand::Resolve { intent, output } => agent_resolve(&intent, output),
        AgentCommand::Policy { output } => agent_policy(output),
    }
}

fn agent_bootstrap(output: OutputFormat) -> lode_core::Result<()> {
    let project_dir = std::env::current_dir().ok().and_then(|p| {
        if p.join(".lode").exists() {
            camino::Utf8PathBuf::from_path_buf(p).ok()
        } else {
            None
        }
    });

    let config = if let Ok(cfg) = lode_core::load_global_config() {
        cfg
    } else {
        lode_core::default_config()
    };

    let info = BootstrapInfo::new(env!("CARGO_PKG_VERSION"), &config, project_dir.as_deref());

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&info)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!("LODE Agent Bootstrap");
        println!("====================");
        println!("Version:     {}", info.lode_version);
        println!("Asset API:   v{}", info.asset_api_version);
        println!("Config:      {}", info.config_path);
        println!(
            "Profile:     {}",
            info.active_profile.as_deref().unwrap_or("(none)")
        );
        println!();
        if let Some(ref project) = info.project {
            println!("Project:     {}", project.name);
            println!("Language:    {}",
                project.language.as_deref().unwrap_or("(detecting)")
            );
        } else {
            println!("No LODE project detected in current directory");
        }
        println!();
        println!("Available assets:");
        println!("  Commands:  {}", info.available_commands);
        println!("  Profiles:  {}", info.available_profiles);
        println!("  Recipes:   {}", info.available_recipes);
        println!("  Templates: {}", info.available_templates);
        println!("  Snippets:  {}", info.available_snippets);
        println!("  Licenses:  {}", info.available_licenses);
        println!();
        println!("Recommended next: {}", info.recommended.next);
    }
    Ok(())
}

fn project_dir() -> lode_core::Result<camino::Utf8PathBuf> {
    let cwd = std::env::current_dir()
        .map_err(|e| LodeError::Message(format!("cannot get current dir: {e}")))?;
    camino::Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| LodeError::Message("non-UTF-8 path".to_string()))
}

fn agent_policy(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let report = generate_agent_policy(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("{}", output::bold("Agent Policy Generated"));
        println!("  Project: {}", report.project_name);
        println!("  Profile: {}", report.profile);
        println!("  Language: {}", report.language);
        for path in &report.files_written {
            println!("  {}  {}", output::green("✔"), path);
        }
        println!(
            "\n{} {} files written",
            output::cyan("ℹ"),
            report.files_written.len()
        );
    }
    Ok(())
}

fn agent_resolve(intent: &str, output: OutputFormat) -> lode_core::Result<()> {
    let config = lode_core::load_global_config().unwrap_or_default();
    let catalog = build_catalog(&config);
    let resolution = resolve_intent(intent, &catalog, &config);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&resolution)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Intent: {intent}");
        println!("=======");
        if let Some(ref profile) = resolution.profile {
            println!("Profile: {profile}");
        }
        if !resolution.recipes.is_empty() {
            println!("Recipes:");
            for r in &resolution.recipes {
                println!("  - {r}");
            }
        }
        if !resolution.commands.is_empty() {
            println!("Commands:");
            for c in &resolution.commands {
                println!("  - {c}");
            }
        }
        if !resolution.templates.is_empty() {
            println!("Templates:");
            for t in &resolution.templates {
                println!("  - {t}");
            }
        }
        println!("Estimated files: {}", resolution.estimated_files);
        println!("Plan ID: {}", resolution.plan_id);
        for w in &resolution.warnings {
            println!("Warning: {w}");
        }
    }
    Ok(())
}
