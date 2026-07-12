use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    error::{LodeError, Result},
    Process, ValidatedRoot,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub description: String,
    pub files: Vec<RecipeFile>,
    pub steps: Vec<RecipeStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeFile {
    pub template: String,
    pub dest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStep {
    pub kind: String,
    pub run: String,
}

pub fn parse_recipe(content: &str) -> Result<Recipe> {
    toml::from_str(content).map_err(|e| LodeError::Message(format!("failed to parse recipe: {e}")))
}

pub fn apply_recipe(
    recipe: &Recipe,
    project_dir: &Path,
    template_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    let root = ValidatedRoot::new(project_dir)?;

    for file in &recipe.files {
        let src = template_dir.join(&file.template);
        root.resolve(&file.dest)?;

        if !src.exists() {
            return Err(LodeError::Message(format!(
                "template not found: {}",
                src.display()
            )));
        }

        if dry_run {
            continue;
        }

        if let Some(parent) = Path::new(&file.dest).parent() {
            root.create_dir_all(parent)?;
        }

        let content = fs::read_to_string(&src).map_err(|source| LodeError::Io {
            path: src.clone(),
            source,
        })?;
        root.write_atomic(&file.dest, content)?;
    }

    for step in &recipe.steps {
        if dry_run {
            continue;
        }
        match step.kind.as_str() {
            "command" => {
                let args = shell_split(&step.run).ok_or_else(|| {
                    LodeError::Message(format!("unable to parse command arguments: {}", step.run))
                })?;
                if args.is_empty() {
                    return Err(LodeError::Message(format!("empty command: {}", step.run)));
                }
                let status = Process::new(&args[0])?
                    .args(&args[1..])
                    .current_dir(project_dir)
                    .status()?;
                if !status.success() {
                    return Err(LodeError::Message(format!(
                        "command '{}' exited with status: {status}",
                        step.run
                    )));
                }
            }
            "mkdir" => {
                root.create_dir_all(&step.run)?;
            }
            "touch" => {
                if let Some(parent) = Path::new(&step.run).parent() {
                    root.create_dir_all(parent)?;
                }
                root.write_atomic(&step.run, "")?;
            }
            other => {
                return Err(LodeError::Message(format!("unknown step kind: {other}")));
            }
        }
    }

    Ok(())
}

pub fn compose_recipes(recipes: &[Recipe]) -> Recipe {
    let name = recipes
        .iter()
        .map(|r| r.name.as_str())
        .collect::<Vec<_>>()
        .join("+");
    let description = recipes
        .iter()
        .map(|r| r.description.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    let mut files = Vec::new();
    let mut steps = Vec::new();
    for recipe in recipes {
        files.extend(recipe.files.clone());
        steps.extend(recipe.steps.clone());
    }
    Recipe {
        name,
        description,
        files,
        steps,
    }
}

pub fn list_recipes(dir: &Path) -> Result<Vec<Recipe>> {
    let mut recipes = Vec::new();
    if !dir.exists() {
        return Ok(recipes);
    }
    for entry in fs::read_dir(dir).map_err(|source| LodeError::Io {
        path: dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| LodeError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "toml" {
                let content = fs::read_to_string(&path).map_err(|source| LodeError::Io {
                    path: path.clone(),
                    source,
                })?;
                if let Ok(recipe) = parse_recipe(&content) {
                    recipes.push(recipe);
                }
            }
        }
    }
    Ok(recipes)
}

/// Split a command string into arguments, respecting single and double quotes.
/// Returns `None` if quotes are unbalanced.
fn shell_split(input: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in input.chars() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if in_single || in_double {
        return None;
    }
    if !current.is_empty() {
        args.push(current);
    }
    Some(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_apply_recipe_dry_run() {
        let temp = tempfile::tempdir().unwrap();
        let template_dir = temp.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(template_dir.join("hello.txt"), "hello world").unwrap();

        let recipe_toml = r#"
name = "test-recipe"
description = "A test recipe"

[[files]]
template = "hello.txt"
dest = "output/hello.txt"

[[steps]]
kind = "mkdir"
run = "output"
"#;
        let recipe = parse_recipe(recipe_toml).unwrap();
        assert_eq!(recipe.name, "test-recipe");
        assert_eq!(recipe.files.len(), 1);
        assert_eq!(recipe.steps.len(), 1);

        apply_recipe(&recipe, temp.path(), &template_dir, true).unwrap();
        assert!(!temp.path().join("output/hello.txt").exists());
    }

    #[test]
    fn compose_recipes_merges() {
        let r1 = Recipe {
            name: "a".into(),
            description: "first".into(),
            files: vec![],
            steps: vec![RecipeStep {
                kind: "touch".into(),
                run: "file1.txt".into(),
            }],
        };
        let r2 = Recipe {
            name: "b".into(),
            description: "second".into(),
            files: vec![],
            steps: vec![RecipeStep {
                kind: "touch".into(),
                run: "file2.txt".into(),
            }],
        };
        let composed = compose_recipes(&[r1, r2]);
        assert_eq!(composed.name, "a+b");
        assert_eq!(composed.steps.len(), 2);
    }

    #[test]
    fn apply_recipe_rejects_destination_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let template_dir = temp.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(template_dir.join("hello.txt"), "hello world").unwrap();

        let recipe = Recipe {
            name: "escape".into(),
            description: String::new(),
            files: vec![RecipeFile {
                template: "hello.txt".into(),
                dest: "../escape.txt".into(),
            }],
            steps: vec![],
        };

        assert!(apply_recipe(&recipe, temp.path(), &template_dir, false).is_err());
        assert!(!temp.path().parent().unwrap().join("escape.txt").exists());
    }
}
