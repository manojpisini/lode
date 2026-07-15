#![deny(unsafe_code)]

use lode_core::cache;

use crate::CacheCommand;
use crate::OutputFormat;

pub(crate) fn cache_command(command: CacheCommand) -> lode_core::Result<()> {
    match command {
        CacheCommand::Stats { output } => cache_stats(output),
        CacheCommand::Clear => cache_clear(),
    }
}

fn cache_stats(output: OutputFormat) -> lode_core::Result<()> {
    let store = cache::cache_stats()?;
    let total = store.entries.len();
    let total_hits: u64 = store.entries.values().map(|e| e.hit_count).sum();
    let table = format!("  entries: {total}\n  hits:    {total_hits}");
    crate::print_output("cache stats", store, output, || table.clone());
    Ok(())
}

fn cache_clear() -> lode_core::Result<()> {
    cache::cache_clear()?;
    println!("  cache cleared");
    Ok(())
}
