use std::collections::BTreeMap;
use std::fs;

use camino::Utf8PathBuf;
use lode_core::{LodeError, ValidatedRoot};

use crate::EnvCommand;

pub(crate) fn env_command(command: EnvCommand) -> lode_core::Result<()> {
    match command {
        EnvCommand::Check => env_check()?,
        EnvCommand::Add {
            key,
            default,
            comment,
            secret,
        } => env_add(&key, default.as_deref(), comment.as_deref(), secret)?,
        EnvCommand::Sync => env_sync()?,
        EnvCommand::Use { profile } => env_use(&profile)?,
    }
    Ok(())
}

fn env_check() -> lode_core::Result<()> {
    let example = read_env_file(".env.example")?;
    let env = read_env_file(".env").unwrap_or_default();
    let missing: Vec<_> = example
        .keys()
        .filter(|key| !env.contains_key(*key))
        .cloned()
        .collect();
    if missing.is_empty() {
        println!("env ok");
        Ok(())
    } else {
        for key in &missing {
            println!("missing {key}");
        }
        Err(LodeError::Message(format!(
            "{} env key(s) missing",
            missing.len()
        )))
    }
}

fn env_add(
    key: &str,
    default: Option<&str>,
    comment: Option<&str>,
    secret: bool,
) -> lode_core::Result<()> {
    let project_dir = crate::current_dir()?;
    let root = ValidatedRoot::new(&project_dir)?;
    let path = project_dir.join(".env.example");
    let mut contents = if path.exists() {
        fs::read_to_string(&path).map_err(|source| LodeError::Io {
            path: path.as_str().into(),
            source,
        })?
    } else {
        String::new()
    };
    if !read_env_entries(&contents).contains_key(key) {
        if !contents.ends_with('\n') && !contents.is_empty() {
            contents.push('\n');
        }
        if let Some(comment) = comment {
            contents.push_str("# ");
            contents.push_str(comment);
            contents.push('\n');
        }
        contents.push_str(key);
        contents.push('=');
        if !secret {
            contents.push_str(default.unwrap_or_default());
        }
        contents.push('\n');
        root.write_atomic(".env.example", contents)?;
    }
    if secret {
        let env_path = project_dir.join(".env");
        let mut env_contents = fs::read_to_string(&env_path).unwrap_or_default();
        if !read_env_entries(&env_contents).contains_key(key) {
            if !env_contents.ends_with('\n') && !env_contents.is_empty() {
                env_contents.push('\n');
            }
            env_contents.push_str(key);
            env_contents.push('=');
            env_contents.push_str(default.unwrap_or_default());
            env_contents.push('\n');
            root.write_atomic(".env", env_contents)?;
        }
    }
    println!("added env key {key}");
    Ok(())
}

fn env_sync() -> lode_core::Result<()> {
    let example = read_env_file(".env.example")?;
    let project_dir = crate::current_dir()?;
    let root = ValidatedRoot::new(&project_dir)?;
    let env_path = project_dir.join(".env");
    let mut env = if env_path.exists() {
        fs::read_to_string(&env_path).map_err(|source| LodeError::Io {
            path: env_path.as_str().into(),
            source,
        })?
    } else {
        String::new()
    };
    let existing = read_env_entries(&env);
    let mut added = 0usize;
    for (key, value) in example {
        if !existing.contains_key(&key) {
            if !env.ends_with('\n') && !env.is_empty() {
                env.push('\n');
            }
            env.push_str(&key);
            env.push('=');
            env.push_str(&value);
            env.push('\n');
            added += 1;
        }
    }
    root.write_atomic(".env", env)?;
    println!("env synced: added {added}");
    Ok(())
}

fn env_use(profile: &str) -> lode_core::Result<()> {
    let project_dir = crate::current_dir()?;
    let root = ValidatedRoot::new(&project_dir)?;
    let source = crate::safe_relative_path(&format!(".env.{profile}"))?;
    let source_path = project_dir.join(&source);
    if !source_path.exists() {
        return Err(LodeError::Message(format!("{source} does not exist")));
    }
    root.copy_file(source, ".env")?;
    println!("env profile active: {profile}");
    Ok(())
}

fn read_env_file(path: &str) -> lode_core::Result<BTreeMap<String, String>> {
    let path = Utf8PathBuf::from(path);
    let contents = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    Ok(read_env_entries(&contents))
}

fn read_env_entries(contents: &str) -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            entries.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    entries
}
