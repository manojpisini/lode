#![deny(unsafe_code)]

use lode_core::{
    build_catalog, build_search_index, default_config, export_catalog, load_search_index,
    save_search_index, AssetCatalogEntry,
};

use crate::output;
use crate::AssetsCommand;
use crate::OutputFormat;

pub(crate) fn assets_command(command: AssetsCommand) -> lode_core::Result<()> {
    match command {
        AssetsCommand::Search {
            query,
            kind,
            status,
            min_quality,
            output,
        } => search_assets(
            &query,
            kind.as_deref(),
            status.as_deref(),
            min_quality,
            output,
        ),
        AssetsCommand::Show { id, output } => show_asset(&id, output),
        AssetsCommand::List { output } => list_assets(output),
        AssetsCommand::Catalog { out } => export_catalog_file(out),
        AssetsCommand::Index {
            rebuild,
            stats,
            output,
        } => index_command(rebuild, stats, output),
    }
}

fn index_command(rebuild: bool, stats: bool, output: OutputFormat) -> lode_core::Result<()> {
    let config = default_config();

    if rebuild {
        let idx = build_search_index(&config)?;
        save_search_index(&idx)?;
        if output.should_use_json() {
            println!(
                "{}",
                serde_json::to_string_pretty(&idx)
                    .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
            );
        } else {
            println!(
                "{}  Built search index: {} entries, {} terms",
                output::green("✔"),
                idx.total_entries,
                idx.word_index.len(),
            );
        }
        return Ok(());
    }

    if stats || !rebuild {
        match load_search_index() {
            Ok(idx) => {
                if output.should_use_json() {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&idx)
                            .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
                    );
                } else {
                    println!("{}  Search Index Status", output::bold("Search Index"));
                    println!(
                        "  {}  {} entries indexed",
                        output::cyan("ℹ"),
                        idx.total_entries
                    );
                    println!(
                        "  {}  {} unique terms",
                        output::cyan("ℹ"),
                        idx.word_index.len()
                    );
                    println!("  {}  Last indexed: {}", output::dim(" "), idx.indexed_at);
                }
            }
            Err(_) => {
                println!(
                    "{}  No search index found. Run `lode assets index --rebuild` to create one.",
                    output::dim("~")
                );
            }
        }
    }
    Ok(())
}

fn search_assets(
    query: &str,
    kind_filter: Option<&str>,
    status_filter: Option<&str>,
    min_quality: Option<u32>,
    output: OutputFormat,
) -> lode_core::Result<()> {
    let config = default_config();

    let results = if let Ok(idx_results) = lode_core::search_index(&lode_core::SearchQuery {
        query: query.to_string(),
        kind: kind_filter.map(|s| s.to_string()),
        status: status_filter.map(|s| s.to_string()),
        min_quality,
        limit: 30,
    }) {
        // Use indexed search
        idx_results
    } else {
        // Fallback to catalog scan
        let catalog = build_catalog(&config);
        let query_lower = query.to_lowercase();

        let mut cat_results: Vec<lode_core::SearchResult> = catalog
            .entries
            .iter()
            .filter(|entry| {
                if let Some(k) = kind_filter {
                    if entry.kind != k {
                        return false;
                    }
                }
                if let Some(s) = status_filter {
                    if entry.status != s {
                        return false;
                    }
                }
                if let Some(mq) = min_quality {
                    if entry.quality_score.unwrap_or(0) < mq {
                        return false;
                    }
                }

                let q = &query_lower;
                entry.summary.to_lowercase().contains(q)
                    || entry.id.to_lowercase().contains(q)
                    || entry.intents.iter().any(|i| i.to_lowercase().contains(q))
                    || entry.tags.iter().any(|t| t.to_lowercase().contains(q))
                    || entry.languages.iter().any(|l| l.to_lowercase().contains(q))
            })
            .map(|entry| lode_core::SearchResult {
                id: entry.id.clone(),
                kind: entry.kind.clone(),
                summary: entry.summary.clone(),
                status: entry.status.clone(),
                maturity: entry.maturity.clone(),
                quality_score: entry.quality_score,
                languages: entry.languages.clone(),
                relevance: relevance_score(entry, query_lower.as_str()),
            })
            .collect();

        cat_results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        cat_results.truncate(30);
        cat_results
    };

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&results)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        if results.is_empty() {
            println!("No assets found matching '{query}'");
            return Ok(());
        }
        println!("Found {} matching assets:\n", results.len());
        for entry in &results {
            let status_tag = match entry.status.as_str() {
                "experimental" => " [exp]",
                "preview" => " [pre]",
                "deprecated" => " [dep]",
                "retired" => " [ret]",
                _ => "",
            };
            println!("  {}{}  {}", entry.id, status_tag, entry.summary);
            if let Some(qs) = entry.quality_score {
                println!("       Quality: {}/100", qs);
            }
            if !entry.languages.is_empty() {
                println!("       Languages: {}", entry.languages.join(", "));
            }
            println!();
        }
    }
    Ok(())
}

fn relevance_score(entry: &AssetCatalogEntry, query: &str) -> f64 {
    let mut score = 0.0;
    if entry.summary.to_lowercase().contains(query) {
        score += 5.0;
    }
    if entry.id.to_lowercase().contains(query) {
        score += 4.0;
    }
    for intent in &entry.intents {
        if intent.to_lowercase().contains(query) {
            score += 2.0;
        }
    }
    for lang in &entry.languages {
        if lang.to_lowercase().contains(query) || query.contains(&lang.to_lowercase()) {
            score += 1.0;
        }
    }
    score
}

fn show_asset(id: &str, output: OutputFormat) -> lode_core::Result<()> {
    let config = default_config();
    let catalog = build_catalog(&config);

    let entry = catalog
        .entries
        .iter()
        .find(|e| e.id == id || e.id.trim_start_matches("profile://") == id);

    match entry {
        Some(e) => {
            if output.should_use_json() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(e)
                        .map_err(|err| lode_core::LodeError::Message(err.to_string()))?
                );
            } else {
                println!("ID:           {}", e.id);
                println!("Kind:         {}", e.kind);
                println!("Summary:      {}", e.summary);
                println!("Status:       {}", e.status);
                println!("Maturity:     {}", e.maturity);
                if let Some(qs) = e.quality_score {
                    println!("Quality:      {}/100", qs);
                }
                if let Some(lv) = &e.last_verified {
                    println!("Last Verified: {}", lv);
                }
                if let Some(vr) = &e.verification_last_result {
                    println!("Verification:  {}", vr);
                }
                if !e.languages.is_empty() && e.languages != ["*"] {
                    println!("Languages: {}", e.languages.join(", "));
                }
                if !e.project_types.is_empty() && e.project_types != ["*"] {
                    println!("Types:    {}", e.project_types.join(", "));
                }
                if !e.intents.is_empty() {
                    println!("Intents:");
                    for intent in &e.intents {
                        println!("  - {intent}");
                    }
                }
                if !e.requires.is_empty() {
                    println!("Requires: {}", e.requires.join(", "));
                }
                if !e.recommends.is_empty() {
                    println!("Recommends: {}", e.recommends.join(", "));
                }
                if e.status == "deprecated" {
                    if let Some(r) = &e.deprecation_replacement {
                        println!("Replacement:  {}", r);
                    }
                    if let Some(r) = &e.deprecation_remove_after {
                        println!("Remove After: {}", r);
                    }
                    if let Some(m) = &e.deprecation_migration {
                        println!("Migration:    {}", m);
                    }
                }
                if !e.verification.is_empty() {
                    println!("Verification:");
                    for v in &e.verification {
                        println!("  - {v}");
                    }
                }
            }
            Ok(())
        }
        None => {
            eprintln!("Asset not found: {id}");
            std::process::exit(1);
        }
    }
}

fn list_assets(output: OutputFormat) -> lode_core::Result<()> {
    let config = default_config();
    let catalog = build_catalog(&config);

    if output.should_use_json() {
        println!(
            "{}",
            serde_json::to_string_pretty(&catalog)
                .map_err(|e| lode_core::LodeError::Message(e.to_string()))?
        );
    } else {
        println!(
            "LODE Asset Catalog ({} total entries)\n",
            catalog.entries.len()
        );

        let mut by_kind: std::collections::BTreeMap<&str, Vec<&AssetCatalogEntry>> =
            std::collections::BTreeMap::new();
        for entry in &catalog.entries {
            by_kind.entry(&entry.kind).or_default().push(entry);
        }

        for (kind, entries) in &by_kind {
            println!("  {} ({})", kind, entries.len());
            for entry in entries.iter().take(10) {
                println!("    {}  {}", entry.id, entry.summary);
            }
            if entries.len() > 10 {
                println!("    ... and {} more", entries.len() - 10);
            }
            println!();
        }
    }
    Ok(())
}

fn export_catalog_file(out: Option<camino::Utf8PathBuf>) -> lode_core::Result<()> {
    let config = default_config();
    let path = match out {
        Some(p) => p,
        None => lode_core::catalog::catalog_path()?,
    };
    export_catalog(&config, &path)?;
    println!("Catalog written to {}", path);
    Ok(())
}
