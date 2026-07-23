use std::collections::HashMap;
use std::fs;

use camino::Utf8PathBuf;
use serde::Serialize;

use crate::pkg::detect_package_manager;

#[derive(Debug, Clone, Serialize)]
pub struct LanguageInfo {
    pub name: String,
    pub confidence: f64,
    pub indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrameworkInfo {
    pub name: String,
    pub language: String,
    pub confidence: f64,
    pub indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdoptionReport {
    pub project_dir: Utf8PathBuf,
    pub project_name: String,
    pub languages: Vec<LanguageInfo>,
    pub frameworks: Vec<FrameworkInfo>,
    pub package_manager: Option<String>,
    pub toolchains: Vec<String>,
    pub recommended_profile: String,
    pub recommended_components: Vec<String>,
    pub detected_assets: Vec<String>,
    pub has_git: bool,
    pub has_git_remote: bool,
    pub git_remote_hint: Option<String>,
    pub missing_recommendations: Vec<String>,
    pub action_plan: Vec<String>,
}

pub fn analyze_project(path: &Utf8PathBuf) -> AdoptionReport {
    let project_name = path
        .file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "project".to_string());

    let manifests = detect_manifests(path);
    let config_files = detect_config_files(path);
    let src_files = detect_source_files(path);
    let has_git = path.join(".git").exists();
    let has_git_remote = detect_git_remote(path);

    let languages = detect_languages(&manifests, &config_files, &src_files);
    let frameworks = detect_frameworks(path, &manifests, &config_files, &src_files, &languages);
    let toolchains = build_toolchains(&languages, &manifests);
    let package_manager = detect_package_manager(path.as_std_path());

    let (recommended_profile, recommended_components) =
        recommend_profile_and_components(&languages, &frameworks, &manifests);

    let detected_assets = build_detected_assets(&languages, &frameworks, &manifests, &toolchains);

    let mut missing_recommendations = Vec::new();
    if !path.join(".editorconfig").exists() {
        missing_recommendations
            .push("add .editorconfig for consistent editor settings".to_string());
    }
    if !path.join(".gitignore").exists() {
        missing_recommendations.push("add .gitignore for version control hygiene".to_string());
    }
    if !path.join("README.md").exists() {
        missing_recommendations.push("add README.md for project documentation".to_string());
    }
    if !path.join("LICENSE").exists() {
        missing_recommendations.push("add LICENSE file for project licensing".to_string());
    }
    if !path.join("CHANGELOG.md").exists() {
        missing_recommendations.push("add CHANGELOG.md for release tracking".to_string());
    }

    let mut action_plan = Vec::new();
    action_plan.push(format!(
        "lode init {project_name} --path {} --profile {recommended_profile}",
        path.parent()
            .map(|p| p.to_string())
            .unwrap_or_else(|| ".".to_string())
    ));
    if !recommended_components.is_empty() {
        action_plan.push(format!("  --with {}", recommended_components.join(",")));
    }
    for lang in &languages {
        action_plan.push(format!("--lang {}", lang.name));
    }
    for rec in &missing_recommendations {
        action_plan.push(rec.clone());
    }
    if !has_git {
        action_plan
            .push("initialize git repository (lode init does this automatically)".to_string());
    }
    if !has_git_remote && has_git {
        action_plan.push("set up a git remote for version control collaboration".to_string());
    }

    let git_remote_hint = if has_git_remote {
        detect_remote_url(path)
    } else {
        None
    };

    AdoptionReport {
        project_dir: path.clone(),
        project_name,
        languages,
        frameworks,
        package_manager,
        toolchains,
        recommended_profile,
        recommended_components,
        detected_assets,
        has_git,
        has_git_remote,
        git_remote_hint,
        missing_recommendations,
        action_plan,
    }
}

fn detect_manifests(path: &Utf8PathBuf) -> Vec<String> {
    let candidates = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
        "go.mod",
        "build.gradle",
        "settings.gradle",
        "pom.xml",
        "Gemfile",
        "CMakeLists.txt",
        "build.zig",
        "Cargo.lock",
    ];
    let mut found: Vec<String> = candidates
        .iter()
        .filter(|f| path.join(f).exists())
        .map(|f| (*f).to_string())
        .collect();

    // Also check some well-known subdirectory manifests
    let subdir_manifests = ["src-tauri/Cargo.toml", "fabric/build.gradle"];
    for m in &subdir_manifests {
        if path.join(m).exists() && !found.iter().any(|f| f == m) {
            found.push(m.to_string());
        }
    }

    found
}

fn detect_config_files(path: &Utf8PathBuf) -> Vec<String> {
    let candidates = [
        "rust-toolchain.toml",
        ".nvmrc",
        ".python-version",
        "tsconfig.json",
        ".editorconfig",
        ".gitignore",
        "Dockerfile",
        "docker-compose.yml",
        "compose.yml",
        ".github/workflows/ci.yml",
        "Makefile",
        "Justfile",
        "manage.py",
        "vite.config.ts",
        "next.config.js",
        "svelte.config.js",
        "astro.config.mjs",
    ];
    candidates
        .iter()
        .filter(|f| path.join(f).exists())
        .map(|f| (*f).to_string())
        .collect()
}

fn detect_source_files(path: &Utf8PathBuf) -> Vec<String> {
    let mut files = Vec::new();
    let scan_dirs = ["src", "app", "lib", "cmd", "problems"];

    for dir in &scan_dirs {
        let dir_path = path.join(dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir_path.as_std_path()) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".rs")
                            || name.ends_with(".py")
                            || name.ends_with(".ts")
                            || name.ends_with(".tsx")
                            || name.ends_with(".js")
                            || name.ends_with(".jsx")
                            || name.ends_with(".go")
                            || name.ends_with(".java")
                            || name.ends_with(".c")
                            || name.ends_with(".cpp")
                            || name.ends_with(".h")
                            || name.ends_with(".hpp")
                            || name.ends_with(".zig")
                            || name.ends_with(".rb")
                            || name.ends_with(".cs")
                            || name.ends_with(".swift")
                        {
                            let relative = format!("{dir}/{name}");
                            if !files.contains(&relative) {
                                files.push(relative);
                            }
                        }
                    }
                }
            }
        }
    }
    files
}

fn detect_languages(
    manifests: &[String],
    config_files: &[String],
    src_files: &[String],
) -> Vec<LanguageInfo> {
    let mut lang_map: HashMap<String, (f64, Vec<String>)> = HashMap::new();

    let manifest_langs: &[(&str, f64, &[&str])] = &[
        ("rust", 0.9, &["Cargo.toml", "Cargo.lock"]),
        ("javascript", 0.7, &["package.json"]),
        ("typescript", 0.7, &["package.json"]),
        ("python", 0.9, &["pyproject.toml", "requirements.txt"]),
        ("go", 0.9, &["go.mod"]),
        ("java", 0.8, &["build.gradle", "settings.gradle", "pom.xml"]),
        ("c", 0.5, &["CMakeLists.txt"]),
        ("cpp", 0.5, &["CMakeLists.txt"]),
        ("zig", 0.9, &["build.zig"]),
        ("ruby", 0.8, &["Gemfile"]),
    ];

    for m in manifests {
        for &(lang, conf, triggers) in manifest_langs {
            if triggers.contains(&m.as_str()) {
                let entry = lang_map
                    .entry(lang.to_string())
                    .or_insert_with(|| (0.0, Vec::new()));
                entry.0 = entry.0.max(conf);
                entry.1.push(format!("manifest: {m}"));
            }
        }
    }

    for cf in config_files {
        match cf.as_str() {
            "rust-toolchain.toml" => update_lang(&mut lang_map, "rust", 0.7, cf),
            ".nvmrc" => update_lang(&mut lang_map, "javascript", 0.5, cf),
            ".python-version" => update_lang(&mut lang_map, "python", 0.5, cf),
            "manage.py" => update_lang(&mut lang_map, "python", 0.6, cf),
            "vite.config.ts" => {
                update_lang(&mut lang_map, "typescript", 0.6, cf);
                update_lang(&mut lang_map, "javascript", 0.4, cf);
            }
            "next.config.js" => {
                update_lang(&mut lang_map, "typescript", 0.5, cf);
                update_lang(&mut lang_map, "javascript", 0.5, cf);
            }
            _ => {}
        }
    }

    for sf in src_files {
        if sf.ends_with(".rs") {
            update_lang(&mut lang_map, "rust", 0.6, sf);
        } else if sf.ends_with(".py") {
            update_lang(&mut lang_map, "python", 0.5, sf);
        } else if sf.ends_with(".ts") || sf.ends_with(".tsx") {
            update_lang(&mut lang_map, "typescript", 0.5, sf);
        } else if sf.ends_with(".js") || sf.ends_with(".jsx") {
            update_lang(&mut lang_map, "javascript", 0.4, sf);
        } else if sf.ends_with(".go") {
            update_lang(&mut lang_map, "go", 0.5, sf);
        } else if sf.ends_with(".java") {
            update_lang(&mut lang_map, "java", 0.5, sf);
        } else if sf.ends_with(".c") {
            update_lang(&mut lang_map, "c", 0.5, sf);
        } else if sf.ends_with(".cpp") {
            update_lang(&mut lang_map, "cpp", 0.5, sf);
        } else if sf.ends_with(".zig") {
            update_lang(&mut lang_map, "zig", 0.5, sf);
        } else if sf.ends_with(".rb") {
            update_lang(&mut lang_map, "ruby", 0.5, sf);
        }
    }

    let mut result: Vec<LanguageInfo> = lang_map
        .into_iter()
        .map(|(name, (confidence, indicators))| LanguageInfo {
            name,
            confidence,
            indicators,
        })
        .collect();
    result.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}

fn update_lang(
    map: &mut HashMap<String, (f64, Vec<String>)>,
    lang: &str,
    confidence: f64,
    indicator: &str,
) {
    let entry = map
        .entry(lang.to_string())
        .or_insert_with(|| (0.0, Vec::new()));
    entry.0 = entry.0.max(confidence);
    entry.1.push(indicator.to_string());
}

fn detect_frameworks(
    project_dir: &Utf8PathBuf,
    manifests: &[String],
    config_files: &[String],
    _src_files: &[String],
    languages: &[LanguageInfo],
) -> Vec<FrameworkInfo> {
    let mut frameworks = Vec::new();

    if config_files.iter().any(|f| f == "manage.py") {
        frameworks.push(FrameworkInfo {
            name: "Django".to_string(),
            language: "python".to_string(),
            confidence: 0.9,
            indicators: vec!["manage.py".to_string()],
        });
    }

    if manifests.iter().any(|f| f == "package.json") {
        let pkg_path = project_dir.join("package.json");
        if let Ok(raw) = fs::read_to_string(pkg_path.as_std_path()) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&raw) {
                let deps = extract_deps(&pkg);
                let js_frameworks: &[(&str, &[&str], &str, f64)] = &[
                    ("React", &["react", "react-dom"], "typescript", 0.9),
                    ("Next.js", &["next"], "typescript", 0.9),
                    ("Vue.js", &["vue"], "javascript", 0.9),
                    ("Svelte", &["svelte"], "javascript", 0.85),
                    ("Astro", &["astro"], "javascript", 0.85),
                    ("Express", &["express"], "javascript", 0.7),
                    ("NestJS", &["@nestjs/core"], "typescript", 0.85),
                    ("Tailwind CSS", &["tailwindcss"], "typescript", 0.6),
                    ("Tauri", &["@tauri-apps/api"], "rust", 0.9),
                    ("Electron", &["electron"], "javascript", 0.8),
                ];
                for &(fw_name, keywords, fw_lang, fw_conf) in js_frameworks {
                    if keywords.iter().any(|k| deps.iter().any(|d| d == k)) {
                        frameworks.push(FrameworkInfo {
                            name: fw_name.to_string(),
                            language: fw_lang.to_string(),
                            confidence: fw_conf,
                            indicators: deps
                                .iter()
                                .filter(|d| keywords.contains(&d.as_str()))
                                .cloned()
                                .collect(),
                        });
                    }
                }
            }
        }
    }

    if manifests.iter().any(|f| f == "Cargo.toml") {
        let cargo_path = project_dir.join("Cargo.toml");
        if let Ok(raw) = fs::read_to_string(cargo_path.as_std_path()) {
            if let Ok(cargo) = toml::from_str::<toml::Value>(&raw) {
                let deps = extract_toml_deps(&cargo);
                let rust_frameworks: &[(&str, &[&str], f64)] = &[
                    ("Tauri", &["tauri"], 0.9),
                    ("Axum", &["axum"], 0.85),
                    ("Actix Web", &["actix-web"], 0.85),
                    ("Rocket", &["rocket"], 0.85),
                    ("Leptos", &["leptos"], 0.85),
                    ("Yew", &["yew"], 0.85),
                    ("Dioxus", &["dioxus"], 0.85),
                    ("Serde", &["serde"], 0.5),
                    ("Tokio", &["tokio"], 0.5),
                    ("Clap", &["clap"], 0.5),
                ];
                for &(fw_name, keywords, fw_conf) in rust_frameworks {
                    if keywords.iter().any(|k| deps.iter().any(|d| d == k)) {
                        frameworks.push(FrameworkInfo {
                            name: fw_name.to_string(),
                            language: "rust".to_string(),
                            confidence: fw_conf,
                            indicators: deps
                                .iter()
                                .filter(|d| keywords.contains(&d.as_str()))
                                .cloned()
                                .collect(),
                        });
                    }
                }
            }
        }
    }

    for lang in languages {
        if lang.name == "python" && manifests.iter().any(|f| f == "pyproject.toml") {
            let pyproject_path = project_dir.join("pyproject.toml");
            if let Ok(raw) = fs::read_to_string(pyproject_path.as_std_path()) {
                if let Ok(pyproject) = toml::from_str::<toml::Value>(&raw) {
                    let deps = extract_toml_deps(&pyproject);
                    let py_frameworks: &[(&str, &[&str], f64)] = &[
                        ("FastAPI", &["fastapi"], 0.9),
                        ("Flask", &["flask"], 0.85),
                        ("Django", &["django"], 0.85),
                        ("Pydantic", &["pydantic"], 0.6),
                        ("SQLAlchemy", &["sqlalchemy"], 0.6),
                    ];
                    for &(fw_name, keywords, fw_conf) in py_frameworks {
                        if keywords.iter().any(|k| deps.iter().any(|d| d == k)) {
                            frameworks.push(FrameworkInfo {
                                name: fw_name.to_string(),
                                language: "python".to_string(),
                                confidence: fw_conf,
                                indicators: deps
                                    .iter()
                                    .filter(|d| keywords.contains(&d.as_str()))
                                    .cloned()
                                    .collect(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Check for Tauri via src-tauri/Cargo.toml
    if !frameworks.iter().any(|f| f.name == "Tauri") {
        let tauri_cargo = project_dir.join("src-tauri/Cargo.toml");
        if tauri_cargo.exists() {
            if let Ok(raw) = fs::read_to_string(tauri_cargo.as_std_path()) {
                if let Ok(cargo) = toml::from_str::<toml::Value>(&raw) {
                    let deps = extract_toml_deps(&cargo);
                    if deps.iter().any(|d| d == "tauri") {
                        frameworks.push(FrameworkInfo {
                            name: "Tauri".to_string(),
                            language: "rust".to_string(),
                            confidence: 0.9,
                            indicators: vec!["src-tauri/Cargo.toml".to_string()],
                        });
                    }
                }
            }
        }
    }

    frameworks.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    frameworks
}

fn extract_deps(pkg: &serde_json::Value) -> Vec<String> {
    let mut deps = Vec::new();
    if let Some(deps_map) = pkg.get("dependencies").and_then(|v| v.as_object()) {
        for key in deps_map.keys() {
            deps.push(key.clone());
        }
    }
    if let Some(deps_map) = pkg.get("devDependencies").and_then(|v| v.as_object()) {
        for key in deps_map.keys() {
            deps.push(key.clone());
        }
    }
    deps
}

fn extract_toml_deps(value: &toml::Value) -> Vec<String> {
    let mut deps = Vec::new();
    if let Some(table) = value.get("dependencies").and_then(|v| v.as_table()) {
        for key in table.keys() {
            deps.push(key.clone());
        }
    }
    if let Some(table) = value.get("dev-dependencies").and_then(|v| v.as_table()) {
        for key in table.keys() {
            deps.push(key.clone());
        }
    }
    if let Some(table) = value.get("build-dependencies").and_then(|v| v.as_table()) {
        for key in table.keys() {
            deps.push(key.clone());
        }
    }
    deps
}

fn recommend_profile_and_components(
    languages: &[LanguageInfo],
    frameworks: &[FrameworkInfo],
    manifests: &[String],
) -> (String, Vec<String>) {
    // Check for special framework-based profiles first
    if frameworks.iter().any(|f| f.name == "Tauri") {
        return ("tauri".to_string(), recommend_rust_components(frameworks));
    }
    if frameworks.iter().any(|f| f.name == "Django") {
        return (
            "django".to_string(),
            recommend_python_components(frameworks),
        );
    }

    let has_ts_src = manifests.iter().any(|m| m == "tsconfig.json");
    let has_package_json = manifests.iter().any(|m| m == "package.json");

    for lang in languages {
        match lang.name.as_str() {
            "rust" => {
                // Check for Tauri first
                if frameworks.iter().any(|f| f.name == "Tauri") {
                    return ("tauri".to_string(), recommend_rust_components(frameworks));
                }
                return ("rust".to_string(), recommend_rust_components(frameworks));
            }
            "python" => {
                if frameworks.iter().any(|f| f.name == "Django") {
                    return (
                        "django".to_string(),
                        recommend_python_components(frameworks),
                    );
                }
                return (
                    "python".to_string(),
                    recommend_python_components(frameworks),
                );
            }
            "typescript" | "javascript" => {
                if has_package_json {
                    for fw in frameworks {
                        match fw.name.as_str() {
                            "Next.js" => {
                                return ("next".to_string(), recommend_js_components(frameworks))
                            }
                            "Svelte" => {
                                return ("svelte".to_string(), recommend_js_components(frameworks))
                            }
                            "Astro" => {
                                return ("astro".to_string(), recommend_js_components(frameworks))
                            }
                            "React" if has_ts_src => {
                                return ("react".to_string(), recommend_js_components(frameworks))
                            }
                            _ => {}
                        }
                    }
                    return ("node".to_string(), recommend_js_components(frameworks));
                }
            }
            "go" => return ("go".to_string(), vec!["ci".to_string()]),
            "java" => return ("java".to_string(), vec!["ci".to_string()]),
            "cpp" => return ("cpp".to_string(), vec![]),
            "c" => return ("c-app".to_string(), vec![]),
            "zig" => return ("zig".to_string(), vec![]),
            "ruby" => return ("ruby".to_string(), vec!["ci".to_string()]),
            _ => {}
        }
    }

    ("core/bare".to_string(), Vec::new())
}

fn recommend_rust_components(frameworks: &[FrameworkInfo]) -> Vec<String> {
    let mut components = vec!["ci".to_string(), "vscode".to_string()];
    if frameworks.iter().any(|f| f.name == "Tauri") {
        components.push("docker".to_string());
    }
    components
}

fn recommend_python_components(_frameworks: &[FrameworkInfo]) -> Vec<String> {
    vec!["ci".to_string(), "vscode".to_string(), "docs".to_string()]
}

fn recommend_js_components(frameworks: &[FrameworkInfo]) -> Vec<String> {
    let mut components = vec!["ci".to_string(), "vscode".to_string()];
    if frameworks.iter().any(|f| f.name == "Docker") || std::path::Path::new("Dockerfile").exists()
    {
        components.push("docker".to_string());
    }
    components
}

fn build_toolchains(languages: &[LanguageInfo], manifests: &[String]) -> Vec<String> {
    let mut toolchains = Vec::new();

    let has_cargo_lock = manifests
        .iter()
        .any(|m| m == "Cargo.lock" || m == "Cargo.toml");
    let has_package_lock = manifests.iter().any(|m| m == "package.json");

    for lang in languages {
        match lang.name.as_str() {
            "rust" if has_cargo_lock => toolchains.push("rustc  (via rustup)".to_string()),
            "javascript" | "typescript" if has_package_lock => {
                toolchains.push("node  (via nvm or fnm)".to_string());
                toolchains.push("npm / pnpm / yarn".to_string());
            }
            "python" => toolchains.push("python  (via pyenv or uv)".to_string()),
            "go" => toolchains.push("go  (via gvm or upstream)".to_string()),
            "java" => toolchains.push("java + gradle / maven".to_string()),
            "cpp" | "c" => toolchains.push("gcc / clang + cmake".to_string()),
            "zig" => toolchains.push("zig  (via zigup)".to_string()),
            "ruby" => toolchains.push("ruby  (via rbenv or rvm)".to_string()),
            _ => {}
        }
    }

    toolchains
}

fn build_detected_assets(
    languages: &[LanguageInfo],
    frameworks: &[FrameworkInfo],
    manifests: &[String],
    toolchains: &[String],
) -> Vec<String> {
    let mut assets = Vec::new();

    for lang in languages {
        assets.push(format!("language/{}", lang.name));
    }
    for fw in frameworks {
        assets.push(format!("framework/{}", fw.name.to_lowercase()));
    }
    for m in manifests {
        assets.push(format!("manifest/{}", m));
    }
    for tc in toolchains {
        let slug = tc.split_whitespace().next().unwrap_or(tc).to_lowercase();
        assets.push(format!("toolchain/{slug}"));
    }

    assets
}

fn detect_git_remote(path: &Utf8PathBuf) -> bool {
    let git_config = path.join(".git").join("config");
    if !git_config.exists() {
        return false;
    }
    if let Ok(raw) = fs::read_to_string(git_config.as_std_path()) {
        raw.contains("[remote")
    } else {
        false
    }
}

fn detect_remote_url(path: &Utf8PathBuf) -> Option<String> {
    let git_config = path.join(".git").join("config");
    if !git_config.exists() {
        return None;
    }
    if let Ok(raw) = fs::read_to_string(git_config.as_std_path()) {
        for line in raw.lines() {
            let trimmed = line.trim();
            if let Some(url) = trimmed.strip_prefix("url = ") {
                return Some(url.trim_matches('"').to_string());
            }
        }
    }
    None
}

pub fn format_adoption_report(report: &AdoptionReport) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Adoption Report: {}", report.project_name));
    lines.push(format!("  Directory: {}", report.project_dir));
    lines.push(String::new());

    // Languages
    lines.push("Languages:".to_string());
    if report.languages.is_empty() {
        lines.push("  (none detected)".to_string());
    } else {
        for lang in &report.languages {
            let pct = (lang.confidence * 100.0) as u32;
            lines.push(format!(
                "  {pct:>3}%  {}  [{}]",
                lang.name,
                lang.indicators.join(", ")
            ));
        }
    }
    lines.push(String::new());

    // Frameworks
    if !report.frameworks.is_empty() {
        lines.push("Frameworks:".to_string());
        for fw in &report.frameworks {
            let pct = (fw.confidence * 100.0) as u32;
            lines.push(format!(
                "  {pct:>3}%  {} ({})  [{}]",
                fw.name,
                fw.language,
                fw.indicators.join(", ")
            ));
        }
        lines.push(String::new());
    }

    // Package manager
    lines.push("Package Manager:".to_string());
    lines.push(format!(
        "  {}",
        report
            .package_manager
            .as_deref()
            .unwrap_or("(none detected)")
    ));
    lines.push(String::new());

    // Toolchains
    if !report.toolchains.is_empty() {
        lines.push("Recommended Toolchains:".to_string());
        for tc in &report.toolchains {
            lines.push(format!("  - {tc}"));
        }
        lines.push(String::new());
    }

    // Git
    lines.push("Version Control:".to_string());
    if report.has_git {
        lines.push("  git: initialized".to_string());
        if report.has_git_remote {
            if let Some(ref url) = report.git_remote_hint {
                lines.push(format!("  remote: {url}"));
            } else {
                lines.push("  remote: configured".to_string());
            }
        } else {
            lines.push("  remote: not configured".to_string());
        }
    } else {
        lines.push("  git: not initialized".to_string());
    }
    lines.push(String::new());

    // Recommendation
    lines.push("Recommended Profile:".to_string());
    lines.push(format!("  {}", report.recommended_profile));
    if !report.recommended_components.is_empty() {
        lines.push(format!(
            "  Components: {}",
            report.recommended_components.join(", ")
        ));
    }
    lines.push(String::new());

    // Missing recommendations
    if !report.missing_recommendations.is_empty() {
        lines.push("Recommendations:".to_string());
        for rec in &report.missing_recommendations {
            lines.push(format!("  - {rec}"));
        }
        lines.push(String::new());
    }

    // Action plan
    lines.push("Action Plan:".to_string());
    for (i, action) in report.action_plan.iter().enumerate() {
        lines.push(format!("  {}. {action}", i + 1));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_directory_detects_no_languages() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let report = analyze_project(&path);
        assert!(report.languages.is_empty());
        assert!(report.package_manager.is_none());
        assert_eq!(report.recommended_profile, "core/bare");
    }

    #[test]
    fn detects_rust_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        fs::write(path.join("Cargo.lock"), "").unwrap();
        let report = analyze_project(&path);
        assert!(report.languages.iter().any(|l| l.name == "rust"));
        assert_eq!(report.package_manager.as_deref(), Some("cargo"));
        assert!(!report.toolchains.is_empty());
    }

    #[test]
    fn detects_node_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(
            path.join("package.json"),
            r#"{"dependencies": {"express": "^4.0.0"}}"#,
        )
        .unwrap();
        fs::write(path.join("package-lock.json"), "").unwrap();
        let report = analyze_project(&path);
        assert!(
            report.languages.iter().any(|l| l.name == "javascript")
                || report.languages.iter().any(|l| l.name == "typescript")
        );
        assert_eq!(report.package_manager.as_deref(), Some("npm"));
    }

    #[test]
    fn detects_nextjs_framework() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(
            path.join("package.json"),
            r#"{"dependencies": {"next": "^14.0.0", "react": "^18.0.0"}}"#,
        )
        .unwrap();
        fs::write(path.join("package-lock.json"), "").unwrap();
        let report = analyze_project(&path);
        assert!(report.frameworks.iter().any(|f| f.name == "Next.js"));
        assert_eq!(report.recommended_profile, "next");
    }

    #[test]
    fn detects_python_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(path.join("pyproject.toml"), "[project]\nname = \"test\"\n").unwrap();
        let report = analyze_project(&path);
        assert!(report.languages.iter().any(|l| l.name == "python"));
        assert!(report.recommended_profile.contains("python"));
    }

    #[test]
    fn detects_django_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(path.join("pyproject.toml"), "[project]\n").unwrap();
        fs::write(path.join("manage.py"), "").unwrap();
        let report = analyze_project(&path);
        assert!(report.frameworks.iter().any(|f| f.name == "Django"));
        assert_eq!(report.recommended_profile, "django");
    }

    #[test]
    fn detects_go_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(path.join("go.mod"), "module test").unwrap();
        let report = analyze_project(&path);
        assert!(report.languages.iter().any(|l| l.name == "go"));
        assert_eq!(report.recommended_profile, "go");
    }

    #[test]
    fn detects_git_repository() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let git_dir = path.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(
            git_dir.join("config"),
            "[core]\n\trepositoryformatversion = 0\n",
        )
        .unwrap();
        let report = analyze_project(&path);
        assert!(report.has_git);
        assert!(!report.has_git_remote);
    }

    #[test]
    fn detects_git_remote() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let git_dir = path.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(
            git_dir.join("config"),
            "[remote \"origin\"]\n\turl = https://github.com/user/repo.git\n",
        )
        .unwrap();
        let report = analyze_project(&path);
        assert!(report.has_git);
        assert!(report.has_git_remote);
        assert_eq!(
            report.git_remote_hint.as_deref(),
            Some("https://github.com/user/repo.git")
        );
    }

    #[test]
    fn detects_source_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(path.join("src/lib.rs"), "pub fn foo() {}").unwrap();
        fs::write(path.join("src/style.css"), "body {}").unwrap();

        let files = detect_source_files(&path);
        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(files.contains(&"src/lib.rs".to_string()));
        assert!(!files.contains(&"src/style.css".to_string()));
    }

    #[test]
    fn manifest_detection_finds_multiple() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        fs::write(path.join("Cargo.toml"), "").unwrap();
        fs::write(path.join("package.json"), "").unwrap();
        let manifests = detect_manifests(&path);
        assert!(manifests.contains(&"Cargo.toml".to_string()));
        assert!(manifests.contains(&"package.json".to_string()));
    }

    #[test]
    fn format_report_is_readable() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let report = analyze_project(&path);
        let formatted = format_adoption_report(&report);
        assert!(formatted.contains("Adoption Report"));
        assert!(formatted.contains("Action Plan"));
    }

    #[test]
    fn tauri_detected_from_src_tauri_cargo() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let tauri_dir = path.join("src-tauri");
        fs::create_dir_all(&tauri_dir).unwrap();
        fs::write(
            tauri_dir.join("Cargo.toml"),
            "[dependencies]\ntauri = \"2\"\n",
        )
        .unwrap();
        fs::write(path.join("package.json"), r#"{"dependencies":{}}"#).unwrap();
        let report = analyze_project(&path);
        assert!(
            report.frameworks.iter().any(|f| f.name == "Tauri"),
            "Tauri should be detected from src-tauri/Cargo.toml"
        );
        assert_eq!(report.recommended_profile, "tauri");
    }

    #[test]
    fn missing_recommendations_for_bare_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let report = analyze_project(&path);
        assert!(!report.missing_recommendations.is_empty());
        assert!(report
            .missing_recommendations
            .iter()
            .any(|r| r.contains(".editorconfig")));
        assert!(report
            .missing_recommendations
            .iter()
            .any(|r| r.contains(".gitignore")));
    }
}
