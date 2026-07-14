#![deny(unsafe_code)]

use std::collections::HashMap;

use lode_core::{
    builtin_asset_deps, find_asset_by_provides, format_dep_graph_dot, format_dep_resolution_table,
    AssetDeps, DepGraphBuilder, LodeError,
};

use crate::{DepGraphCommand, OutputFormat};

pub(crate) fn depgraph_command(command: DepGraphCommand) -> lode_core::Result<()> {
    match command {
        DepGraphCommand::List { output } => depgraph_list(output),
        DepGraphCommand::Show { id, output } => depgraph_show(&id, output),
        DepGraphCommand::Check { root, output } => depgraph_check(&root, output),
        DepGraphCommand::Dot { root, out } => depgraph_dot(&root, out),
    }
}

fn build_resolver(extra_deps: Option<HashMap<String, AssetDeps>>) -> DepGraphBuilder {
    let mut builder = DepGraphBuilder::new();
    let builtins = builtin_asset_deps();

    // Add all built-in deps
    for (id, deps) in &builtins {
        builder.add_asset(id, deps.clone());
    }

    // Add extra deps (e.g., from project.toml or plugins)
    if let Some(extras) = extra_deps {
        for (id, deps) in &extras {
            builder.add_asset(id, deps.clone());
        }
    }

    builder
}

fn depgraph_list(output: OutputFormat) -> lode_core::Result<()> {
    let builtins = builtin_asset_deps();

    let mut assets: Vec<(&String, &AssetDeps)> = builtins.iter().collect();
    assets.sort_by(|a, b| a.0.cmp(b.0));

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&builtins)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Built-in assets ({} total):", assets.len());
        println!();
        for (id, deps) in &assets {
            let req_count = deps.requires.len();
            let con_count = deps.conflicts.len();
            let rec_count = deps.recommends.len();
            println!(
                "  {id}  [requires={req_count}, conflicts={con_count}, recommends={rec_count}]"
            );
            if !deps.provides.is_empty() {
                println!("       provides: [{}]", deps.provides.join(", "));
            }
        }
    }
    Ok(())
}

fn depgraph_show(id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let builtins = builtin_asset_deps();

    let deps = builtins
        .get(id)
        .ok_or_else(|| LodeError::Message(format!("asset not found: {id}")))?;

    // Resolve just this asset
    let builder = build_resolver(None);
    let resolution = builder.resolve(vec![id.to_string()]);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&resolution)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("Asset: {id}");
        println!();

        if !deps.requires.is_empty() {
            println!("  Requires:");
            for req in &deps.requires {
                let ver = req
                    .version
                    .as_deref()
                    .map(|v| format!(" v{v}"))
                    .unwrap_or_default();
                println!("    - {}{ver}", req.id);
            }
        } else {
            println!("  Requires: (none)");
        }

        if !deps.conflicts.is_empty() {
            println!("  Conflicts:");
            for con in &deps.conflicts {
                println!("    - ! {}", con.id);
            }
        }

        if !deps.recommends.is_empty() {
            println!("  Recommends:");
            for rec in &deps.recommends {
                println!("    - {}", rec.id);
            }
        }

        if !deps.provides.is_empty() {
            println!("  Provides:");
            for p in &deps.provides {
                println!("    - {p}");
            }
        }

        println!();

        if resolution.conflicts.is_empty() && resolution.graph.cycles.is_empty() {
            println!("  Status: no conflicts or cycles detected");
        } else {
            if !resolution.conflicts.is_empty() {
                println!("  Conflicts detected:");
                for c in &resolution.conflicts {
                    println!("    ! {a} <-> {b}", a = c.asset_a, b = c.asset_b);
                }
            }
            if !resolution.graph.cycles.is_empty() {
                println!("  Cycles detected:");
                for cycle in &resolution.graph.cycles {
                    println!("    ~ {}", cycle.join(" -> "));
                }
            }
        }

        // Show which assets provide capabilities this asset needs
        let mut unmet = Vec::new();
        for req in &deps.requires {
            let providers = find_asset_by_provides(&builtins, &req.id);
            if !providers.is_empty() {
                for p in providers {
                    if *p != id {
                        unmet.push((req.id.clone(), (*p).clone()));
                    }
                }
            }
        }
        if !unmet.is_empty() {
            println!();
            println!("  Capability mappings:");
            for (capability, provider) in &unmet {
                println!("    \"{capability}\" is provided by {provider}");
            }
        }
    }
    Ok(())
}

fn depgraph_check(roots: &[String], output: OutputFormat) -> lode_core::Result<()> {
    let builder = build_resolver(None);

    let root_ids = if roots.is_empty() {
        // Use all built-in assets as roots
        let builtins = builtin_asset_deps();
        builtins.keys().cloned().collect()
    } else {
        roots.to_vec()
    };

    let resolution = builder.resolve(root_ids);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&resolution)
                .map_err(|e| LodeError::Message(e.to_string()))?
        );
    } else {
        println!("{}", format_dep_resolution_table(&resolution));
    }

    // Return error exit code if there are issues
    if !resolution.errors.is_empty() || !resolution.conflicts.is_empty() {
        Err(LodeError::Message("dependency graph has issues".to_string()))
    } else {
        Ok(())
    }
}

fn depgraph_dot(roots: &[String], out: Option<camino::Utf8PathBuf>) -> lode_core::Result<()> {
    let builder = build_resolver(None);

    let root_ids = if roots.is_empty() {
        let builtins = builtin_asset_deps();
        builtins.keys().cloned().collect()
    } else {
        roots.to_vec()
    };

    let resolution = builder.resolve(root_ids);
    let dot = format_dep_graph_dot(&resolution.graph);

    if let Some(path) = out {
        std::fs::write(path.as_std_path(), &dot)
            .map_err(|e| LodeError::Message(format!("failed to write DOT file: {e}")))?;
    } else {
        println!("{dot}");
    }

    Ok(())
}
