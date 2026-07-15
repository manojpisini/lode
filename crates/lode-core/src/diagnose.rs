use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisPattern {
    pub id: String,
    pub label: String,
    pub trigger: Vec<String>,
    pub severity: String,
    pub suggestion: String,
    pub recipe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    pub pattern: String,
    pub label: String,
    pub severity: String,
    pub suggestion: String,
    pub recipe: Option<String>,
}

fn builtin_patterns() -> Vec<DiagnosisPattern> {
    vec![
        DiagnosisPattern {
            id: "missing-module".to_string(),
            label: "Missing module or import".to_string(),
            trigger: vec![
                "error[E0432]".to_string(),
                "error[E0583]".to_string(),
                "failed to resolve".to_string(),
                "cannot find module".to_string(),
                "ModuleNotFoundError".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Add the missing module declaration or install the dependency".to_string(),
            recipe: Some("dependency/add".to_string()),
        },
        DiagnosisPattern {
            id: "type-mismatch".to_string(),
            label: "Type mismatch".to_string(),
            trigger: vec![
                "error[E0308]".to_string(),
                "mismatched types".to_string(),
                "TypeError".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Check the expected and actual types, add an explicit conversion or cast".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "borrow-checker".to_string(),
            label: "Borrow checker violation".to_string(),
            trigger: vec![
                "error[E0499]".to_string(),
                "error[E0502]".to_string(),
                "cannot borrow".to_string(),
                "cannot move".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Restructure ownership: clone, use references with correct lifetimes, or restructure to avoid conflicting borrows".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "missing-semicolon".to_string(),
            label: "Missing semicolon".to_string(),
            trigger: vec!["error[E0574]".to_string(), "expected ;".to_string(), "expected semicolon".to_string()],
            severity: "error".to_string(),
            suggestion: "Add a semicolon at the end of the statement".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "unused-variable".to_string(),
            label: "Unused variable".to_string(),
            trigger: vec!["warning: unused variable".to_string(), "unused import".to_string()],
            severity: "warning".to_string(),
            suggestion: "Prefix the variable with an underscore or remove it".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "undefined-name".to_string(),
            label: "Undefined name".to_string(),
            trigger: vec![
                "error[E0425]".to_string(),
                "cannot find value".to_string(),
                "NameError".to_string(),
                "is not defined".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Define the name or import it from the correct module".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "lifetime-annotation".to_string(),
            label: "Missing lifetime annotation".to_string(),
            trigger: vec![
                "error[E0106]".to_string(),
                "missing lifetime specifier".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Add an explicit lifetime parameter to the function signature".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "cfg-missing".to_string(),
            label: "Missing feature gate or cfg".to_string(),
            trigger: vec![
                "error[E0635]".to_string(),
                "configuration option".to_string(),
                "unstable feature".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Enable the required feature flag or cfg attribute".to_string(),
            recipe: Some("feature/enable".to_string()),
        },
        DiagnosisPattern {
            id: "test-failure".to_string(),
            label: "Test assertion failure".to_string(),
            trigger: vec![
                "assertion failed".to_string(),
                "panicked at".to_string(),
                "test failed".to_string(),
                "FAILED".to_string(),
            ],
            severity: "error".to_string(),
            suggestion: "Review the failing assertion and fix the logic or expected value".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "clippy-lint".to_string(),
            label: "Clippy lint warning".to_string(),
            trigger: vec!["warning: `".to_string(), "clippy::".to_string()],
            severity: "warning".to_string(),
            suggestion: "Follow the clippy suggestion or add an allow attribute if intentional".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "deprecated".to_string(),
            label: "Deprecated API usage".to_string(),
            trigger: vec!["warning: use of deprecated".to_string(), "is deprecated".to_string()],
            severity: "warning".to_string(),
            suggestion: "Replace with the recommended alternative from the deprecation notice".to_string(),
            recipe: None,
        },
        DiagnosisPattern {
            id: "dead-code".to_string(),
            label: "Dead code".to_string(),
            trigger: vec!["warning: function is never used".to_string(), "warning: struct is never constructed".to_string(), "dead_code".to_string()],
            severity: "warning".to_string(),
            suggestion: "Remove the unused code or add an allow(dead_code) attribute".to_string(),
            recipe: None,
        },
    ]
}

pub fn diagnose_output(output: &str) -> Vec<Diagnosis> {
    let patterns = builtin_patterns();
    let mut results = Vec::new();
    for pattern in &patterns {
        if pattern.trigger.iter().any(|t| output.contains(t.as_str())) {
            results.push(Diagnosis {
                pattern: pattern.id.clone(),
                label: pattern.label.clone(),
                severity: pattern.severity.clone(),
                suggestion: pattern.suggestion.clone(),
                recipe: pattern.recipe.clone(),
            });
        }
    }
    results
}

pub fn list_diagnosis_patterns() -> Vec<DiagnosisPattern> {
    builtin_patterns()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnose_matches_rust_errors() {
        let output = "error[E0308]: mismatched types\nexpected i32, found String";
        let results = diagnose_output(output);
        assert!(results.iter().any(|d| d.pattern == "type-mismatch"));
    }

    #[test]
    fn test_diagnose_matches_borrow_checker() {
        let output = "error[E0502]: cannot borrow `x` as mutable";
        let results = diagnose_output(output);
        assert!(results.iter().any(|d| d.pattern == "borrow-checker"));
    }

    #[test]
    fn test_diagnose_empty_output() {
        let results = diagnose_output("everything is fine");
        assert!(results.is_empty());
    }

    #[test]
    fn test_list_patterns_returns_12() {
        let patterns = list_diagnosis_patterns();
        assert_eq!(patterns.len(), 12);
    }
}
