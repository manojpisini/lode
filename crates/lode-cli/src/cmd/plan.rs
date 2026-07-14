#![deny(unsafe_code)]

use lode_core::{
    build_catalog, default_config, resolve_intent, LodeError, Plan, PlanStatus,
};

use crate::PlanCommand;
use crate::OutputFormat;

pub(crate) fn plan_command(command: PlanCommand) -> lode_core::Result<()> {
    match command {
        PlanCommand::Create { intent, output } => plan_create(&intent, output),
        PlanCommand::Show { plan_id, output } => plan_show(&plan_id, output),
        PlanCommand::Validate { plan_id, output } => plan_validate(&plan_id, output),
        PlanCommand::Apply { plan_id, dry_run, output } => plan_apply(&plan_id, dry_run, output),
        PlanCommand::Rollback { plan_id, dry_run, output } => plan_rollback(&plan_id, dry_run, output),
        PlanCommand::List { output } => plan_list(output),
    }
}

fn project_dir() -> lode_core::Result<camino::Utf8PathBuf> {
    let cwd = std::env::current_dir().map_err(|e| {
        LodeError::Message(format!("cannot get current dir: {e}"))
    })?;
    camino::Utf8PathBuf::from_path_buf(cwd).map_err(|_| {
        LodeError::Message("non-UTF-8 path".to_string())
    })
}

fn plan_create(intent: &str, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let config = default_config();
    let catalog = build_catalog(&config);
    let resolution = resolve_intent(intent, &catalog, &config);

    let mut plan = Plan::new(intent);
    plan.profile = resolution.profile.clone();
    plan.metadata.source_profile = resolution.profile.clone();
    plan.metadata.source_recipes = resolution.recipes.clone();
    plan.metadata.source_commands = resolution.commands.clone();
    plan.metadata.estimated_files = resolution.estimated_files;

    // Add operations from resolution
    if let Some(ref profile) = resolution.profile {
        plan.add_operation(lode_core::Operation::RunCommand {
            command: format!("lode profile use {profile}"),
            description: format!("activate profile {profile}"),
        });
    }
    for cmd in &resolution.commands {
        plan.add_operation(lode_core::Operation::RunMacro {
            name: cmd.clone(),
            args: std::collections::HashMap::new(),
        });
    }
    for recipe in &resolution.recipes {
        plan.add_operation(lode_core::Operation::ApplyRecipe {
            name: recipe.clone(),
        });
    }

    plan.status = PlanStatus::Pending;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&plan)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Plan: {}", plan.plan_id);
        println!("Intents: {}", plan.intent);
        if let Some(ref p) = plan.profile {
            println!("Profile: {p}");
        }
        println!("Operations: {}", plan.operations.len());
        for op in &plan.operations {
            println!("  - {}", op.description());
        }
        if !resolution.warnings.is_empty() {
            println!("\nWarnings:");
            for w in &resolution.warnings {
                println!("  - {w}");
            }
        }
        println!("\nRun `lode plan validate {}` to validate", plan.plan_id);
        println!("Run `lode plan apply {}` to apply", plan.plan_id);
        println!("Run `lode plan apply {} --dry-run` for preview", plan.plan_id);
    }

    plan.save(&dir)?;
    Ok(())
}

fn plan_show(plan_id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let plan = Plan::load(&dir, plan_id)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&plan)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Plan: {}", plan.plan_id);
        println!("Intent: {}", plan.intent);
        println!("Status: {:?}", plan.status);
        println!("Created: {}", plan.created_at);
        if let Some(ref p) = plan.profile {
            println!("Profile: {p}");
        }
        println!("\nOperations:");
        for op in &plan.operations {
            println!("  - {}", op.description());
        }
        println!("\nRollback operations: {}", plan.rollback_ops.len());
        println!("Verification gates: {}", plan.verification.len());
        println!("\nMetadata:");
        println!("  Estimated files: {}", plan.metadata.estimated_files);
        if !plan.metadata.source_recipes.is_empty() {
            println!("  Source recipes: {}", plan.metadata.source_recipes.join(", "));
        }
        if !plan.metadata.source_commands.is_empty() {
            println!("  Source commands: {}", plan.metadata.source_commands.join(", "));
        }
    }
    Ok(())
}

fn plan_validate(plan_id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let plan = Plan::load(&dir, plan_id)?;
    let validation = plan.validate(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&validation)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if validation.valid {
            println!("Plan {} is valid", plan_id);
        } else {
            println!("Plan {} has errors:", plan_id);
            for e in &validation.errors {
                println!("  - {e}");
            }
        }
        if !validation.warnings.is_empty() {
            println!("\nWarnings:");
            for w in &validation.warnings {
                println!("  - {w}");
            }
        }
    }
    Ok(())
}

fn plan_apply(plan_id: &str, dry_run: bool, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let plan = Plan::load(&dir, plan_id)?;

    if dry_run {
        println!("[DRY RUN] Would apply plan {plan_id}");
        for op in &plan.operations {
            println!("  {}", op.description());
        }
        return Ok(());
    }

    let report = plan.apply(&dir, false)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Applied plan {plan_id}:");
        if !report.created.is_empty() {
            println!("  Created: {} files", report.created.len());
        }
        if !report.modified.is_empty() {
            println!("  Modified: {} files", report.modified.len());
        }
        if !report.deleted.is_empty() {
            println!("  Deleted: {} files", report.deleted.len());
        }
        if !report.errors.is_empty() {
            println!("\nErrors:");
            for e in &report.errors {
                println!("  - {e}");
            }
        }
        if !report.warnings.is_empty() {
            println!("\nWarnings:");
            for w in &report.warnings {
                println!("  - {w}");
            }
        }
        println!("\nStatus: {}", report.status);
    }
    Ok(())
}

fn plan_rollback(plan_id: &str, dry_run: bool, output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let plan = Plan::load(&dir, plan_id)?;

    if dry_run {
        println!("[DRY RUN] Would rollback plan {plan_id}");
        for op in &plan.rollback_ops {
            println!("  {}", op.description());
        }
        return Ok(());
    }

    let report = plan.rollback(&dir, false)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Rolled back plan {plan_id}:");
        if !report.created.is_empty() {
            println!("  Created: {}", report.created.len());
        }
        if !report.modified.is_empty() {
            println!("  Modified: {}", report.modified.len());
        }
        if !report.deleted.is_empty() {
            println!("  Deleted: {}", report.deleted.len());
        }
        println!("\nStatus: {}", report.status);
    }
    Ok(())
}

fn plan_list(output: OutputFormat) -> lode_core::Result<()> {
    let dir = project_dir()?;
    let plans = Plan::list(&dir)?;

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&plans)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        if plans.is_empty() {
            println!("No plans found");
            return Ok(());
        }
        println!("Plans ({}):", plans.len());
        for plan_id in &plans {
            let plan = Plan::load(&dir, plan_id).ok();
            if let Some(p) = plan {
                println!("  {}  {:?}  {}", plan_id, p.status, p.intent);
            } else {
                println!("  {plan_id}");
            }
        }
    }
    Ok(())
}
