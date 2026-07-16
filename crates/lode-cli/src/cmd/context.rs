#![deny(unsafe_code)]

use lode_core::{compile_context, load_global_config, ContextPack, LodeError};

use crate::output;
use crate::ContextCommand;
use crate::OutputFormat;

pub(crate) fn context_command(command: ContextCommand) -> lode_core::Result<()> {
    match command {
        ContextCommand::Build { output } => context_build(output),
        ContextCommand::Show { output } => context_show(output),
        ContextCommand::Diff => context_diff(),
        ContextCommand::Verify => context_verify(),
        ContextCommand::Compile { budget, output } => context_compile(budget, output),
    }
}

fn project_dir() -> lode_core::Result<camino::Utf8PathBuf> {
    let cwd = std::env::current_dir()
        .map_err(|e| LodeError::Message(format!("cannot get current dir: {e}")))?;
    camino::Utf8PathBuf::from_path_buf(cwd)
        .map_err(|_| LodeError::Message("non-UTF-8 path".to_string()))
}

fn context_build(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let pack = ContextPack::generate(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&pack).map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Context pack built for {}", pack.project_name);
        println!("  Files tracked: {}", pack.files.len());
        println!("  Decisions: {}", pack.decisions.len());
        println!("  Quality gates: {}", pack.quality_gates.len());
        println!("  Dependencies: {}", pack.dependencies.len());
        println!("  Recent changes: {}", pack.recent_changes.len());
        println!("\nContext files written to _ctx_/");
    }
    Ok(())
}

fn context_show(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let pack = ContextPack::from_project(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&pack).map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Context for {}:\n", pack.project_name);
        if !pack.files.is_empty() {
            println!("Context files:");
            for f in &pack.files {
                println!("  {} — {}", f.path, f.summary);
            }
            println!();
        }
        if !pack.decisions.is_empty() {
            println!("Active decisions:");
            for d in &pack.decisions {
                println!(
                    "  [{}] {} ({})",
                    if d.status == "accepted" { "x" } else { " " },
                    d.title,
                    d.status
                );
            }
            println!();
        }
        if !pack.quality_gates.is_empty() {
            println!("Quality gates:");
            for g in &pack.quality_gates {
                println!("  {}  required={}", g.name, g.required);
            }
            println!();
        }
        if !pack.dependencies.is_empty() {
            println!("Dependencies:");
            for d in &pack.dependencies {
                println!("  {} {} ({})", d.name, d.version, d.kind);
            }
        }
    }
    Ok(())
}

fn context_diff() -> lode_core::Result<()> {
    let dir = project_dir()?;
    let pack = ContextPack::from_project(&dir)?;

    println!("Context diff for {}", pack.project_name);
    println!("\nTracked files:");
    for f in &pack.files {
        println!("  {} ({:.8})", f.path, f.hash);
    }
    println!(
        "\n{} decisions, {} gates, {} deps",
        pack.decisions.len(),
        pack.quality_gates.len(),
        pack.dependencies.len(),
    );
    Ok(())
}

fn context_compile(budget: Option<usize>, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let resolved_budget = budget.or_else(|| {
        load_global_config()
            .ok()
            .map(|config| config.preferences.agents.context_budget_tokens as usize)
    });

    let report = compile_context(&dir, resolved_budget)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("{}", output::bold("Context Compiled"));
        println!(
            "  {} {} / {} tokens used (budget: {})",
            output::green("✔"),
            report.total_estimated_tokens.min(report.budget_tokens),
            report.total_estimated_tokens,
            report.budget_tokens,
        );
        println!(
            "  {} {} files included, {} skipped",
            output::cyan("ℹ"),
            report.included_files,
            report.skipped_files,
        );
        println!("  {} {}", output::dim("Output:"), report.output_path,);
    }
    Ok(())
}

fn context_verify() -> lode_core::Result<()> {
    let dir = project_dir()?;
    let ctx_dir = dir.join("_ctx_");
    let expected = [
        "CONTEXT_INDEX.md",
        "PROJECT_SUMMARY.md",
        "CURRENT_STATE.md",
        "ARCHITECTURE_MAP.md",
        "QUALITY_GATES.md",
        "ACTIVE_DECISIONS.md",
        "OPEN_RISKS.md",
        "RECENT_CHANGES.md",
    ];

    let mut all_ok = true;
    for name in &expected {
        let path = ctx_dir.join(name);
        if path.exists() {
            println!("  OK  {name}");
        } else {
            println!("  MISSING  {name}");
            all_ok = false;
        }
    }

    if all_ok {
        println!("\nAll context files present.");
    } else {
        println!("\nRun `lode context build` to generate missing files.");
    }
    Ok(())
}
