#![deny(unsafe_code)]

use std::fs;

use camino::Utf8PathBuf;
use lode_core::{default_config, load_global_config, save_global_config, LodeError, SCHEMA_VERSION};

use crate::{ConfigCommand, OutputFormat};

pub(crate) fn config_command(command: ConfigCommand) -> lode_core::Result<()> {
    match command {
        ConfigCommand::Show {
            format,
            defaults,
            project,
            section,
        } => {
            if defaults && project {
                return Err(LodeError::Message(
                    "--defaults and --project cannot be used together".to_string(),
                ));
            }
            let value = if project {
                load_project_config_value()?
            } else if defaults {
                toml::Value::try_from(default_config())?
            } else {
                toml::Value::try_from(load_global_config()?)?
            };
            let value = config_section_value(value, section.as_deref())?;
            match format {
                OutputFormat::Toml => println!("{}", toml::to_string_pretty(&value)?),
                OutputFormat::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .map_err(|error| LodeError::Message(error.to_string()))?
                ),
            }
        }
        ConfigCommand::Validate { defaults, project } => {
            if defaults && project {
                return Err(LodeError::Message(
                    "--defaults and --project cannot be used together".to_string(),
                ));
            }
            if project {
                let value = load_project_config_value()?;
                validate_config_value_schema(&value)?;
                println!("project config valid");
            } else if defaults {
                let value = toml::Value::try_from(default_config())?;
                validate_config_value_schema(&value)?;
                println!("default config valid");
            } else {
                load_global_config()?;
                println!("config valid");
            }
        }
        ConfigCommand::Diff => {
            let default = default_config();
            let global = load_global_config()?;
            print_config_diff(&default, &global)?;
        }
        ConfigCommand::Set { key, value } => {
            let mut config = load_global_config()?;
            set_config_value(&mut config, &key, &value)?;
            save_global_config(&config)?;
            println!("set {key} = {value}");
        }
        ConfigCommand::Reset { key } => {
            let mut config = load_global_config()?;
            let value = default_config_value(&key)?;
            set_config_value(&mut config, &key, &value)?;
            save_global_config(&config)?;
            println!("reset {key}");
        }
        ConfigCommand::Edit { key } => {
            let global_config_path = lode_core::global_config_path()?;
            let value = load_global_config()?;
            let current = match key.as_str() {
                "identity.author" => value.identity.author,
                "identity.email" => value.identity.email,
                "identity.org" => value.identity.org,
                "identity.license" => value.identity.license,
                "convention.default_case" => value.convention.default_case,
                "git.initial_branch" => value.git.initial_branch,
                "git.auto_init" => value.git.auto_init.to_string(),
                "git.initial_commit" => value.git.initial_commit.to_string(),
                "git.initial_commit_msg" => value.git.initial_commit_msg,
                _ => {
                    return Err(LodeError::Message(format!(
                        "unsupported config key: {key}"
                    )))
                }
            };
            println!("Current value for {key}: {current}");
            println!("Config file: {global_config_path}");
            println!("Use `lode config set {key} <value>` to change it");
        }
    }
    Ok(())
}

fn default_config_value(key: &str) -> lode_core::Result<String> {
    let config = default_config();
    let value = match key {
        "identity.author" => config.identity.author,
        "identity.email" => config.identity.email,
        "identity.org" => config.identity.org,
        "identity.license" => config.identity.license,
        "convention.default_case" => config.convention.default_case,
        "git.initial_branch" => config.git.initial_branch,
        "git.auto_init" => config.git.auto_init.to_string(),
        "git.initial_commit" => config.git.initial_commit.to_string(),
        "git.initial_commit_msg" => config.git.initial_commit_msg,
        _ => return Err(LodeError::Message(format!("unsupported config key: {key}"))),
    };
    Ok(value)
}

fn load_project_config_value() -> lode_core::Result<toml::Value> {
    let path = Utf8PathBuf::from(".lode").join("project.toml");
    let raw = fs::read_to_string(&path).map_err(|source| LodeError::Io {
        path: path.as_str().into(),
        source,
    })?;
    toml::from_str(&raw).map_err(|source| LodeError::TomlDeserialize {
        path: path.as_str().into(),
        source: Box::new(source),
    })
}

fn config_section_value(
    value: toml::Value,
    section: Option<&str>,
) -> lode_core::Result<toml::Value> {
    let Some(section) = section else {
        return Ok(value);
    };
    value
        .get(section)
        .cloned()
        .ok_or_else(|| LodeError::Message(format!("unknown config section: {section}")))
}

fn validate_config_value_schema(value: &toml::Value) -> lode_core::Result<()> {
    let found = value
        .get("schema_version")
        .and_then(toml::Value::as_integer)
        .ok_or_else(|| LodeError::Message("missing schema_version".to_string()))?;
    if found == i64::from(SCHEMA_VERSION) {
        Ok(())
    } else {
        Err(LodeError::SchemaMismatch {
            expected: SCHEMA_VERSION,
            found: u32::try_from(found).map_err(|_| {
                LodeError::Message(format!(
                    "schema_version {found} is out of valid range for u32"
                ))
            })?,
        })
    }
}

fn print_config_diff(
    default: &lode_core::LodeConfig,
    global: &lode_core::LodeConfig,
) -> lode_core::Result<()> {
    let default_value = toml::Value::try_from(default)?;
    let global_value = toml::Value::try_from(global)?;
    let mut changes = Vec::new();
    diff_toml("", &default_value, &global_value, &mut changes);
    if changes.is_empty() {
        println!("config diff: no changes from defaults");
    } else {
        for change in changes {
            println!("{change}");
        }
    }
    Ok(())
}

fn diff_toml(prefix: &str, left: &toml::Value, right: &toml::Value, changes: &mut Vec<String>) {
    match (left, right) {
        (toml::Value::Table(left), toml::Value::Table(right)) => {
            for (key, right_value) in right {
                let next = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                if let Some(left_value) = left.get(key) {
                    diff_toml(&next, left_value, right_value, changes);
                } else {
                    changes.push(format!("+ {next} = {right_value}"));
                }
            }
        }
        _ if left != right => changes.push(format!("~ {prefix}: {left} -> {right}")),
        _ => {}
    }
}

fn set_config_value(
    config: &mut lode_core::LodeConfig,
    key: &str,
    value: &str,
) -> lode_core::Result<()> {
    match key {
        "identity.author" => config.identity.author = value.to_string(),
        "identity.email" => config.identity.email = value.to_string(),
        "identity.org" => config.identity.org = value.to_string(),
        "identity.license" => config.identity.license = value.to_string(),
        "convention.default_case" => {
            if !matches!(
                value,
                "snake_case" | "kebab-case" | "camelCase" | "PascalCase"
            ) {
                return Err(LodeError::Message(format!(
                    "unsupported convention.default_case: {value}"
                )));
            }
            config.convention.default_case = value.to_string();
        }
        "git.initial_branch" => config.git.initial_branch = value.to_string(),
        "git.auto_init" => config.git.auto_init = parse_bool(value)?,
        "git.initial_commit" => config.git.initial_commit = parse_bool(value)?,
        "git.initial_commit_msg" => config.git.initial_commit_msg = value.to_string(),
        _ => {
            return Err(LodeError::Message(format!(
                "unsupported config key: {key}"
            )))
        }
    }
    Ok(())
}

fn parse_bool(value: &str) -> lode_core::Result<bool> {
    match value {
        "true" | "yes" | "1" => Ok(true),
        "false" | "no" | "0" => Ok(false),
        _ => Err(LodeError::Message(format!("expected boolean, got {value}"))),
    }
}
