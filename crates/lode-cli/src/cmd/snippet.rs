#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{global_asset_dir, LodeError, ValidatedRoot};

use crate::{
    list_dir, open_editor, safe_relative_path, write_validated_output, OutputFormat, SnippetAsset,
    SnippetCommand,
};

pub(crate) fn snippet_command(command: SnippetCommand) -> lode_core::Result<()> {
    match command {
        SnippetCommand::List { lang, output } => {
            let root = global_asset_dir("snippets")?;
            if output.should_use_json() {
                let mut snippets = Vec::new();
                if let Some(lang) = lang {
                    collect_snippet_assets(&root.join(lang), &mut snippets)?;
                } else {
                    collect_snippet_assets(&root, &mut snippets)?;
                }
                let values = snippets
                    .into_iter()
                    .map(|snippet| {
                        serde_json::json!({
                            "lang": snippet.lang,
                            "name": snippet.name,
                            "path": snippet.path,
                        })
                    })
                    .collect::<Vec<_>>();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&values)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                );
            } else if let Some(lang) = lang {
                list_dir(root.join(lang))?;
            } else {
                list_dir(root)?;
            }
        }
        SnippetCommand::Show { name, lang } => {
            let path = resolve_snippet_path(&name, lang.as_deref())?;
            print!(
                "{}",
                fs::read_to_string(&path).map_err(|source| LodeError::Io {
                    path: path.as_str().into(),
                    source,
                })?
            );
        }
        SnippetCommand::Search { query } => {
            search_snippets(&query)?;
        }
        SnippetCommand::Add {
            name,
            lang,
            trigger,
            desc,
        } => {
            add_snippet(&name, &lang, trigger.as_deref(), desc.as_deref())?;
        }
        SnippetCommand::Remove { name, lang } => {
            remove_snippet(&name, lang.as_deref())?;
        }
        SnippetCommand::Insert {
            name,
            file,
            lang,
            line,
        } => {
            insert_snippet(&name, lang.as_deref(), file, line)?;
        }
        SnippetCommand::Export { lang, format, out } => {
            export_snippets(lang.as_deref(), &format, out)?;
        }
        SnippetCommand::Edit { name } => {
            let path = resolve_snippet_path(&name, None)?;
            open_editor(&path)?;
        }
    }
    Ok(())
}

pub(crate) fn add_snippet(
    name: &str,
    lang: &str,
    trigger: Option<&str>,
    desc: Option<&str>,
) -> lode_core::Result<()> {
    let relative = safe_relative_path(&format!("{lang}/{name}.snippet"))?;
    let asset_dir = global_asset_dir("snippets")?;
    let root = ValidatedRoot::new(&asset_dir)?;
    let path = asset_dir.join(&relative);
    if path.exists() {
        return Err(LodeError::Message(format!(
            "snippet already exists: {name}"
        )));
    }
    if let Some(parent) = relative.parent() {
        root.create_dir_all(parent)?;
    }
    let trigger = trigger.unwrap_or(name);
    let desc = desc.unwrap_or("User snippet");
    let contents = format!(
        "name: {name}\nlang: {lang}\ntrigger: {trigger}\ndescription: {desc}\n---\n{trigger} $1\n"
    );
    root.write_atomic(relative, contents)?;
    println!("created snippet {lang}/{name}");
    Ok(())
}

pub(crate) fn remove_snippet(name: &str, lang: Option<&str>) -> lode_core::Result<()> {
    let path = resolve_snippet_path(name, lang)?;
    let asset_dir = global_asset_dir("snippets")?;
    let root = ValidatedRoot::new(&asset_dir)?;
    let relative = path.strip_prefix(&asset_dir).map_err(|_| {
        LodeError::Message(format!("snippet path is outside the global root: {path}"))
    })?;
    root.remove_file(relative)?;
    println!("removed snippet {name}");
    Ok(())
}

pub(crate) fn insert_snippet(
    name: &str,
    lang: Option<&str>,
    file: Option<Utf8PathBuf>,
    line: Option<usize>,
) -> lode_core::Result<()> {
    let path = resolve_snippet_path(name, lang)?;
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    let snippet = parse_snippet_asset(&path, &raw);
    if let Some(file) = file {
        let existing = fs::read_to_string(&file).unwrap_or_default();
        let updated = insert_text_at_line(&existing, &snippet.body, line.unwrap_or(usize::MAX));
        write_validated_output(&file, updated)?;
        println!("inserted snippet {name} into {file}");
    } else {
        print!("{}", snippet.body);
    }
    Ok(())
}

fn insert_text_at_line(existing: &str, snippet: &str, line: usize) -> String {
    let mut lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    let snippet_lines = snippet.lines().map(str::to_string).collect::<Vec<_>>();
    let index = if line == 0 || line == usize::MAX {
        lines.len()
    } else {
        line.saturating_sub(1).min(lines.len())
    };
    lines.splice(index..index, snippet_lines);
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

pub(crate) fn resolve_snippet_path(
    name: &str,
    lang: Option<&str>,
) -> lode_core::Result<Utf8PathBuf> {
    let root = global_asset_dir("snippets")?;
    if let Some(lang) = lang {
        let relative = safe_relative_path(&format!("{lang}/{name}.snippet"))?;
        let path = root.join(relative);
        if path.exists() {
            return Ok(path);
        }
        return Err(LodeError::Message(format!(
            "snippet not found: {lang}/{name}"
        )));
    }

    let mut matches = Vec::new();
    collect_snippet_named(&root, name, &mut matches)?;
    match matches.len() {
        0 => Err(LodeError::Message(format!("snippet not found: {name}"))),
        1 => Ok(matches.remove(0)),
        _ => Err(LodeError::Message(format!(
            "snippet name is ambiguous; pass --lang for {name}"
        ))),
    }
}

fn collect_snippet_named(
    path: &Utf8PathBuf,
    name: &str,
    matches: &mut Vec<Utf8PathBuf>,
) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            collect_snippet_named(&child, name, matches)?;
        }
        return Ok(());
    }
    if path.file_stem() == Some(name) && path.extension() == Some("snippet") {
        matches.push(path.clone());
    }
    Ok(())
}

pub(crate) fn search_snippets(query: &str) -> lode_core::Result<()> {
    let root = global_asset_dir("snippets")?;
    let mut matches = Vec::new();
    collect_snippet_matches(&root, query, &mut matches)?;
    for path in matches {
        println!("{path}");
    }
    Ok(())
}

fn collect_snippet_matches(
    path: &Utf8PathBuf,
    query: &str,
    matches: &mut Vec<Utf8PathBuf>,
) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            collect_snippet_matches(&child, query, matches)?;
        }
        return Ok(());
    }
    let haystack = format!(
        "{}\n{}",
        path.as_str(),
        fs::read_to_string(path).unwrap_or_default()
    )
    .to_ascii_lowercase();
    if haystack.contains(&query.to_ascii_lowercase()) {
        matches.push(path.clone());
    }
    Ok(())
}

pub(crate) fn export_snippets(
    lang: Option<&str>,
    format: &str,
    out: Option<Utf8PathBuf>,
) -> lode_core::Result<()> {
    let root = global_asset_dir("snippets")?;
    let scan_root = lang.map_or(root.clone(), |lang| root.join(lang));
    let mut snippets = Vec::new();
    collect_snippet_assets(&scan_root, &mut snippets)?;
    snippets.sort_by(|left, right| {
        left.lang
            .cmp(&right.lang)
            .then_with(|| left.name.cmp(&right.name))
    });

    let rendered = match format {
        "vscode" | "json" => render_vscode_snippets(&snippets)?,
        "zed" => render_zed_snippets(&snippets)?,
        "neovim" | "nvim" => render_neovim_snippets(&snippets),
        "jetbrains" | "intellij" => render_jetbrains_snippets(&snippets),
        "markdown" | "md" => render_markdown_snippets(&snippets),
        "plain" | "text" => render_plain_snippets(&snippets),
        other => {
            return Err(LodeError::Message(format!(
                "unsupported snippet export format: {other}"
            )))
        }
    };

    if let Some(path) = out {
        write_validated_output(&path, rendered)?;
        println!("exported {} snippets to {path}", snippets.len());
    } else {
        print!("{rendered}");
    }
    Ok(())
}

pub(crate) fn collect_snippet_assets(
    path: &Utf8PathBuf,
    snippets: &mut Vec<SnippetAsset>,
) -> lode_core::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })? {
            let entry = entry.map_err(|source| LodeError::Io {
                path: path.as_str().into(),
                source,
            })?;
            let child = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                LodeError::Message(format!("path is not valid UTF-8: {}", path.display()))
            })?;
            collect_snippet_assets(&child, snippets)?;
        }
        return Ok(());
    }
    if path.extension() != Some("snippet") {
        return Ok(());
    }
    let contents = fs::read_to_string(path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    snippets.push(parse_snippet_asset(path, &contents));
    Ok(())
}

fn parse_snippet_asset(path: &Utf8PathBuf, contents: &str) -> SnippetAsset {
    let mut name = path
        .file_stem()
        .map(str::to_string)
        .unwrap_or_else(|| "snippet".to_string());
    let mut lang = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(str::to_string)
        .unwrap_or_else(|| "any".to_string());
    let mut body = contents.to_string();

    if let Some((header, raw_body)) = contents.split_once("---") {
        body = raw_body.trim_start_matches(['\r', '\n']).to_string();
        for line in header.lines() {
            if let Some((key, value)) = line.split_once(':') {
                match key.trim() {
                    "name" => name = value.trim().to_string(),
                    "lang" => lang = value.trim().to_string(),
                    _ => {}
                }
            }
        }
    }

    SnippetAsset {
        lang,
        name,
        body,
        path: path.clone(),
    }
}

fn render_vscode_snippets(snippets: &[SnippetAsset]) -> lode_core::Result<String> {
    let mut output = serde_json::Map::new();
    for snippet in snippets {
        let key = format!("{}:{}", snippet.lang, snippet.name);
        output.insert(
            key,
            serde_json::json!({
                "prefix": snippet.name,
                "scope": snippet.lang,
                "body": snippet.body.lines().collect::<Vec<_>>(),
                "description": format!("Lode snippet from {}", snippet.path),
            }),
        );
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(output))
        .map_err(|error| LodeError::Message(error.to_string()))
}

fn render_zed_snippets(snippets: &[SnippetAsset]) -> lode_core::Result<String> {
    let mut output = serde_json::Map::new();
    for snippet in snippets {
        output.insert(
            format!("{}:{}", snippet.lang, snippet.name),
            serde_json::json!({
                "prefix": snippet.name,
                "body": snippet.body,
                "description": format!("Lode snippet from {}", snippet.path),
            }),
        );
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(output))
        .map_err(|error| LodeError::Message(error.to_string()))
}

fn render_neovim_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::from("return {\n");
    for snippet in snippets {
        output.push_str(&format!(
            "  {{ lang = {:?}, trigger = {:?}, body = {:?} }},\n",
            snippet.lang, snippet.name, snippet.body
        ));
    }
    output.push_str("}\n");
    output
}

fn render_jetbrains_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::from("<templateSet group=\"Lode\">\n");
    for snippet in snippets {
        output.push_str(&format!(
            "  <template name=\"{}\" value=\"{}\" description=\"{} snippet\" toReformat=\"true\" toShortenFQNames=\"true\" />\n",
            xml_escape(&snippet.name),
            xml_escape(&snippet.body),
            xml_escape(&snippet.lang)
        ));
    }
    output.push_str("</templateSet>\n");
    output
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_markdown_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::from("# Lode Snippets\n\n");
    for snippet in snippets {
        output.push_str(&format!(
            "## {} / {}\n\nSource: `{}`\n\n```{}\n{}```\n\n",
            snippet.lang, snippet.name, snippet.path, snippet.lang, snippet.body
        ));
    }
    output
}

pub(crate) fn render_plain_snippets(snippets: &[SnippetAsset]) -> String {
    let mut output = String::new();
    for snippet in snippets {
        output.push_str(&format!(
            "[{}:{}]\n{}\n",
            snippet.lang, snippet.name, snippet.body
        ));
    }
    output
}
