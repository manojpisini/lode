use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDependency {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl AssetDependency {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDeps {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<AssetDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<AssetDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommends: Vec<AssetDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provides: Vec<String>,
}

impl AssetDeps {
    pub fn new() -> Self {
        Self {
            requires: Vec::new(),
            conflicts: Vec::new(),
            recommends: Vec::new(),
            provides: Vec::new(),
        }
    }

    pub fn requires(mut self, deps: Vec<AssetDependency>) -> Self {
        self.requires = deps;
        self
    }

    pub fn conflicts(mut self, deps: Vec<AssetDependency>) -> Self {
        self.conflicts = deps;
        self
    }

    pub fn recommends(mut self, deps: Vec<AssetDependency>) -> Self {
        self.recommends = deps;
        self
    }

    pub fn provides(mut self, ids: Vec<String>) -> Self {
        self.provides = ids;
        self
    }
}

impl Default for AssetDeps {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DepGraphNode {
    pub id: String,
    pub depth: usize,
    pub resolved: bool,
    pub provides: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DepEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetResolution {
    pub id: String,
    pub depth: usize,
    pub requires: Vec<String>,
    pub required_by: Vec<String>,
    pub provides: Vec<String>,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DepConflict {
    pub asset_a: String,
    pub asset_b: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DepGraphNode>,
    pub edges: Vec<DepEdge>,
    pub resolutions: Vec<AssetResolution>,
    pub conflicts: Vec<DepConflict>,
    pub cycles: Vec<Vec<String>>,
}

impl DependencyGraph {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DepResolution {
    pub roots: Vec<String>,
    pub resolution: Vec<AssetResolution>,
    pub unresolved: Vec<String>,
    pub conflicts: Vec<DepConflict>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub graph: DependencyGraph,
}

pub struct DepGraphBuilder {
    adj: HashMap<String, Vec<String>>,
    rev_adj: HashMap<String, Vec<String>>,
    recommends_map: HashMap<String, Vec<String>>,
    conflicts_map: HashMap<String, Vec<String>>,
    provides_map: HashMap<String, Vec<String>>,
    dep_decls: HashMap<String, AssetDeps>,
}

impl DepGraphBuilder {
    pub fn new() -> Self {
        Self {
            adj: HashMap::new(),
            rev_adj: HashMap::new(),
            recommends_map: HashMap::new(),
            conflicts_map: HashMap::new(),
            provides_map: HashMap::new(),
            dep_decls: HashMap::new(),
        }
    }

    pub fn add_asset(&mut self, id: &str, deps: AssetDeps) {
        self.dep_decls.insert(id.to_string(), deps.clone());

        for req in &deps.requires {
            self.adj
                .entry(id.to_string())
                .or_default()
                .push(req.id.clone());
            self.rev_adj
                .entry(req.id.clone())
                .or_default()
                .push(id.to_string());
        }

        for rec in &deps.recommends {
            self.recommends_map
                .entry(id.to_string())
                .or_default()
                .push(rec.id.clone());
        }

        for con in &deps.conflicts {
            self.conflicts_map
                .entry(id.to_string())
                .or_default()
                .push(con.id.clone());
            self.conflicts_map
                .entry(con.id.clone())
                .or_default()
                .push(id.to_string());
        }

        self.provides_map
            .entry(id.to_string())
            .or_default()
            .extend(deps.provides.clone());
    }

    pub fn add_root_dep(&mut self, id: &str) {
        self.adj.entry(id.to_string()).or_default();
        self.rev_adj.entry(id.to_string()).or_default();
    }

    fn collect_transitive_closure(&self, roots: &[String]) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for root in roots {
            if visited.insert(root.clone()) {
                queue.push_back(root.clone());
            }
        }

        while let Some(id) = queue.pop_front() {
            if let Some(deps) = self.adj.get(&id) {
                for dep in deps {
                    if visited.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        let mut result: Vec<String> = visited.into_iter().collect();
        result.sort();
        result
    }

    fn detect_cycles(&self, nodes: &[String]) -> Vec<Vec<String>> {
        let node_set: HashSet<&str> = nodes.iter().map(|s| s.as_str()).collect();
        let mut cycles = Vec::new();

        let mut visited: HashSet<&str> = HashSet::new();
        let mut in_stack: HashSet<&str> = HashSet::new();
        let mut stack: Vec<&str> = Vec::new();

        fn dfs<'a>(
            node: &'a str,
            adj: &'a HashMap<String, Vec<String>>,
            node_set: &'a HashSet<&'a str>,
            visited: &mut HashSet<&'a str>,
            in_stack: &mut HashSet<&'a str>,
            stack: &mut Vec<&'a str>,
            cycles: &mut Vec<Vec<String>>,
        ) {
            visited.insert(node);
            in_stack.insert(node);
            stack.push(node);

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    if !node_set.contains(neighbor.as_str()) {
                        continue;
                    }
                    if !visited.contains(neighbor.as_str()) {
                        dfs(neighbor, adj, node_set, visited, in_stack, stack, cycles);
                    } else if in_stack.contains(neighbor.as_str()) {
                        let cycle_start = stack
                            .iter()
                            .position(|n| *n == neighbor.as_str())
                            .unwrap_or(0);
                        let cycle: Vec<String> = stack[cycle_start..]
                            .iter()
                            .map(|s| (*s).to_string())
                            .collect();
                        cycles.push(cycle);
                    }
                }
            }

            stack.pop();
            in_stack.remove(node);
        }

        for node in nodes {
            if !visited.contains(node.as_str()) {
                dfs(
                    node,
                    &self.adj,
                    &node_set,
                    &mut visited,
                    &mut in_stack,
                    &mut stack,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn detect_conflicts(&self, nodes: &[String]) -> Vec<DepConflict> {
        let mut conflicts = Vec::new();
        let node_set: HashSet<&str> = nodes.iter().map(|s| s.as_str()).collect();

        let mut checked: HashSet<(String, String)> = HashSet::new();

        for node in nodes {
            if let Some(conflict_ids) = self.conflicts_map.get(node) {
                for conflict in conflict_ids {
                    if !node_set.contains(conflict.as_str()) {
                        continue;
                    }
                    let key = if node < conflict {
                        (node.clone(), conflict.clone())
                    } else {
                        (conflict.clone(), node.clone())
                    };
                    if checked.insert(key) {
                        conflicts.push(DepConflict {
                            asset_a: node.clone(),
                            asset_b: conflict.clone(),
                            description: format!("{node} conflicts with {conflict}"),
                        });
                    }
                }
            }
        }

        conflicts
    }

    fn topological_sort(&self, nodes: &[String]) -> Vec<AssetResolution> {
        let node_set: HashSet<&str> = nodes.iter().map(|s| s.as_str()).collect();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();

        for node in nodes {
            in_degree.entry(node).or_insert(0);
        }

        for node in nodes {
            if let Some(neighbors) = self.adj.get(node) {
                for neighbor in neighbors {
                    if node_set.contains(neighbor.as_str()) {
                        *in_degree.entry(neighbor.as_str()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut queue: VecDeque<&str> = VecDeque::new();
        for (node, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node);
            }
        }

        let mut resolved_order: Vec<&str> = Vec::new();
        while let Some(node) = queue.pop_front() {
            resolved_order.push(node);
            if let Some(neighbors) = self.adj.get(node) {
                for neighbor in neighbors {
                    if node_set.contains(neighbor.as_str()) {
                        if let Some(degree) = in_degree.get_mut(neighbor.as_str()) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }
            }
        }

        let resolved_set: HashSet<&str> = resolved_order.iter().copied().collect();
        let mut unresolved: Vec<String> = Vec::new();
        for node in nodes {
            if !resolved_set.contains(node.as_str()) {
                unresolved.push(node.clone());
            }
        }

        let mut depth_map: HashMap<&str, usize> = HashMap::new();
        for node in resolved_order.iter().copied().rev() {
            let depth = if let Some(neighbors) = self.adj.get(node) {
                neighbors
                    .iter()
                    .filter(|n| node_set.contains(n.as_str()))
                    .filter_map(|n| depth_map.get(n.as_str()))
                    .max()
                    .map(|d| d + 1)
                    .unwrap_or(0)
            } else {
                0
            };
            depth_map.insert(node, depth);
        }

        let mut resolutions: Vec<AssetResolution> = resolved_order
            .iter()
            .enumerate()
            .map(|(_, id)| {
                let requires = self
                    .adj
                    .get(*id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|n| node_set.contains(n.as_str()))
                    .collect();
                let required_by = self
                    .rev_adj
                    .get(*id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|n| node_set.contains(n.as_str()))
                    .collect();
                let provides = self.provides_map.get(*id).cloned().unwrap_or_default();
                AssetResolution {
                    id: (*id).to_string(),
                    depth: depth_map.get(id).copied().unwrap_or(0),
                    requires,
                    required_by,
                    provides,
                    resolved: true,
                }
            })
            .collect();

        for id in &unresolved {
            let requires = self
                .adj
                .get(id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|n| node_set.contains(n.as_str()))
                .collect();
            let required_by = self
                .rev_adj
                .get(id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|n| node_set.contains(n.as_str()))
                .collect();
            let provides = self.provides_map.get(id).cloned().unwrap_or_default();
            resolutions.push(AssetResolution {
                id: id.clone(),
                depth: 0,
                requires,
                required_by,
                provides,
                resolved: false,
            });
        }

        resolutions
    }

    pub fn resolve(self, roots: Vec<String>) -> DepResolution {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate root assets exist in the graph
        let all_ids: HashSet<&str> = self.dep_decls.keys().map(|s| s.as_str()).collect();
        let mut validated_roots = Vec::new();

        for root in &roots {
            if all_ids.contains(root.as_str()) || self.adj.contains_key(root) {
                validated_roots.push(root.clone());
            } else {
                errors.push(format!("asset not found in graph: {root}"));
            }
        }

        // Collect transitive closure
        let all_nodes = self.collect_transitive_closure(&validated_roots);

        // Check for missing dependencies
        for node in &all_nodes {
            if let Some(deps) = self.dep_decls.get(node) {
                for req in &deps.requires {
                    if !all_nodes.contains(&req.id) && !all_ids.contains(req.id.as_str()) {
                        warnings.push(format!(
                            "{node} requires {id} which is not in the graph",
                            id = req.id
                        ));
                    }
                }
                for rec in &deps.recommends {
                    if !all_nodes.contains(&rec.id) {
                        warnings.push(format!(
                            "{node} recommends {id} which is not installed",
                            id = rec.id
                        ));
                    }
                }
            }
        }

        // Detect cycles
        let cycles = self.detect_cycles(&all_nodes);
        for cycle in &cycles {
            warnings.push(format!("dependency cycle detected: {}", cycle.join(" -> ")));
        }

        // Detect conflicts
        let conflicts = self.detect_conflicts(&all_nodes);

        // Topological sort
        let resolution = self.topological_sort(&all_nodes);

        let unresolved: Vec<String> = resolution
            .iter()
            .filter(|r| !r.resolved)
            .map(|r| r.id.clone())
            .collect();

        // Build graph structure
        let nodes: Vec<DepGraphNode> = resolution
            .iter()
            .map(|r| {
                let provides = self.provides_map.get(&r.id).cloned().unwrap_or_default();
                DepGraphNode {
                    id: r.id.clone(),
                    depth: r.depth,
                    resolved: r.resolved,
                    provides,
                }
            })
            .collect();

        let mut edges = Vec::new();
        for node in &all_nodes {
            if let Some(deps) = self.adj.get(node) {
                for dep in deps {
                    if all_nodes.contains(dep) {
                        edges.push(DepEdge {
                            from: node.clone(),
                            to: dep.clone(),
                            kind: "requires".to_string(),
                        });
                    }
                }
            }
            if let Some(recs) = self.recommends_map.get(node) {
                for rec in recs {
                    if all_nodes.contains(rec) {
                        edges.push(DepEdge {
                            from: node.clone(),
                            to: rec.clone(),
                            kind: "recommends".to_string(),
                        });
                    }
                }
            }
        }

        let graph = DependencyGraph {
            nodes,
            edges,
            resolutions: resolution.clone(),
            conflicts: conflicts.clone(),
            cycles,
        };

        DepResolution {
            roots: validated_roots,
            resolution,
            unresolved,
            conflicts,
            warnings,
            errors,
            graph,
        }
    }
}

impl Default for DepGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn builtin_asset_deps() -> HashMap<String, AssetDeps> {
    let mut map: HashMap<String, AssetDeps> = HashMap::new();

    // Core profiles
    map.insert(
        "profile/core/bare".to_string(),
        AssetDeps::new().requires(vec![AssetDependency::new("template/core/dotfiles")]),
    );
    map.insert(
        "profile/rust".to_string(),
        AssetDeps::new()
            .requires(vec![AssetDependency::new("profile/core/bare")])
            .recommends(vec![AssetDependency::new("component/ci")]),
    );
    map.insert(
        "profile/python".to_string(),
        AssetDeps::new()
            .requires(vec![AssetDependency::new("profile/core/bare")])
            .recommends(vec![AssetDependency::new("component/ci")]),
    );
    map.insert(
        "profile/node".to_string(),
        AssetDeps::new()
            .requires(vec![AssetDependency::new("profile/core/bare")])
            .recommends(vec![AssetDependency::new("component/ci")]),
    );

    // Components
    map.insert(
        "component/ci".to_string(),
        AssetDeps::new().requires(vec![AssetDependency::new("template/github/workflows")]),
    );
    map.insert(
        "component/security".to_string(),
        AssetDeps::new()
            .requires(vec![AssetDependency::new("template/github/workflows")])
            .conflicts(vec![AssetDependency::new("component/ci")]),
    );
    map.insert(
        "component/release".to_string(),
        AssetDeps::new().requires(vec![AssetDependency::new("template/github/workflows")]),
    );
    map.insert(
        "component/docker".to_string(),
        AssetDeps::new().requires(vec![AssetDependency::new("template/docker")]),
    );
    map.insert(
        "component/devcontainer".to_string(),
        AssetDeps::new().requires(vec![AssetDependency::new("template/devcontainer")]),
    );

    // Template groups
    map.insert(
        "template/core/dotfiles".to_string(),
        AssetDeps::new().provides(vec![
            "file/.gitignore".to_string(),
            "file/.gitattributes".to_string(),
            "file/.editorconfig".to_string(),
            "file/.env.example".to_string(),
        ]),
    );
    map.insert(
        "template/github/workflows".to_string(),
        AssetDeps::new().provides(vec!["ci/platform/github".to_string()]),
    );
    map.insert(
        "template/docker".to_string(),
        AssetDeps::new().provides(vec![
            "file/Dockerfile".to_string(),
            "file/compose.yml".to_string(),
        ]),
    );
    map.insert(
        "template/devcontainer".to_string(),
        AssetDeps::new().provides(vec!["config/devcontainer".to_string()]),
    );

    map
}

pub fn find_asset_by_provides<'a>(
    deps_map: &'a HashMap<String, AssetDeps>,
    capability: &str,
) -> Vec<&'a String> {
    deps_map
        .iter()
        .filter(|(_, deps)| deps.provides.iter().any(|p| p == capability))
        .map(|(id, _)| id)
        .collect()
}

pub fn format_dep_resolution_table(resolution: &DepResolution) -> String {
    let mut lines = Vec::new();

    lines.push("Dependency Resolution".to_string());
    lines.push(format!("  Roots: {}", resolution.roots.join(", ")));

    if resolution.unresolved.is_empty() {
        lines.push("  Status: fully resolved".to_string());
    } else {
        lines.push(format!(
            "  Status: {} unresolved",
            resolution.unresolved.len()
        ));
    }

    if !resolution.conflicts.is_empty() {
        lines.push(format!("  Conflicts: {}", resolution.conflicts.len()));
        for c in &resolution.conflicts {
            lines.push(format!("    ! {} <-> {}", c.asset_a, c.asset_b));
        }
    }

    if !resolution.warnings.is_empty() {
        lines.push(format!("  Warnings: {}", resolution.warnings.len()));
        for w in &resolution.warnings {
            lines.push(format!("    - {w}"));
        }
    }

    if !resolution.errors.is_empty() {
        lines.push(format!("  Errors: {}", resolution.errors.len()));
        for e in &resolution.errors {
            lines.push(format!("    !! {e}"));
        }
    }

    lines.push(String::new());
    lines.push("Resolution Order:".to_string());

    for (i, r) in resolution.resolution.iter().enumerate() {
        let status = if r.resolved { "OK" } else { "!!" };
        lines.push(format!(
            "  {i:>3}. [{status}] {} (depth={}, requires=[{}], required_by=[{}])",
            r.id,
            r.depth,
            r.requires.join(", "),
            r.required_by.join(", "),
        ));
        if !r.provides.is_empty() {
            lines.push(format!("       provides: [{}]", r.provides.join(", ")));
        }
    }

    if resolution.errors.is_empty()
        && resolution.conflicts.is_empty()
        && resolution.unresolved.is_empty()
    {
        lines.push(String::new());
        lines.push("✓ All dependencies resolved without conflicts.".to_string());
    } else {
        lines.push(String::new());
        lines.push("✗ Resolution has issues (see above).".to_string());
    }

    lines.join("\n")
}

pub fn format_dep_graph_dot(graph: &DependencyGraph) -> String {
    let mut lines = Vec::new();
    lines.push("digraph lode_deps {".to_string());
    lines.push("  rankdir=LR;".to_string());
    lines.push("  node [shape=box, style=rounded];".to_string());

    for node in &graph.nodes {
        let color = if node.resolved { "lightgreen" } else { "coral" };
        lines.push(format!(
            "  \"{}\" [fillcolor={}, style=filled, label=\"{}\\n(depth {})\"];",
            node.id, color, node.id, node.depth
        ));
    }

    for edge in &graph.edges {
        let color = match edge.kind.as_str() {
            "recommends" => "dashed",
            _ => "solid",
        };
        lines.push(format!(
            "  \"{}\" -> \"{}\" [style={}, label=\"{}\"];",
            edge.from, edge.to, color, edge.kind
        ));
    }

    for conflict in &graph.conflicts {
        lines.push(format!(
            "  \"{}\" -> \"{}\" [color=red, style=dotted, label=\"conflict\"];",
            conflict.asset_a, conflict.asset_b
        ));
    }

    for cycle in &graph.cycles {
        for window in cycle.windows(2) {
            lines.push(format!(
                "  \"{}\" -> \"{}\" [color=purple, style=bold, label=\"cycle\"];",
                window[0], window[1]
            ));
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_resolves_cleanly() {
        let builder = DepGraphBuilder::new();
        let resolution = builder.resolve(vec![]);
        assert!(resolution.resolution.is_empty());
        assert!(resolution.errors.is_empty());
        assert!(resolution.conflicts.is_empty());
    }

    #[test]
    fn single_asset_with_no_deps() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset("my-asset", AssetDeps::new());
        let resolution = builder.resolve(vec!["my-asset".to_string()]);
        assert_eq!(resolution.resolution.len(), 1);
        assert!(resolution.resolution[0].resolved);
        assert_eq!(resolution.resolution[0].id, "my-asset");
        assert!(resolution.errors.is_empty());
    }

    #[test]
    fn simple_chain_resolves_in_order() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "app",
            AssetDeps::new().requires(vec![AssetDependency::new("lib")]),
        );
        builder.add_asset(
            "lib",
            AssetDeps::new().requires(vec![AssetDependency::new("base")]),
        );
        builder.add_asset("base", AssetDeps::new());

        let resolution = builder.resolve(vec!["app".to_string()]);
        assert_eq!(resolution.resolution.len(), 3);
        assert!(resolution.errors.is_empty());

        let depths: HashMap<&str, usize> = resolution
            .resolution
            .iter()
            .map(|r| (r.id.as_str(), r.depth))
            .collect();
        assert_eq!(*depths.get("base").unwrap(), 0);
        assert_eq!(*depths.get("lib").unwrap(), 1);
        assert_eq!(*depths.get("app").unwrap(), 2);
    }

    #[test]
    fn conflict_detected() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "a",
            AssetDeps::new().conflicts(vec![AssetDependency::new("b")]),
        );
        builder.add_asset("b", AssetDeps::new());

        let resolution = builder.resolve(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(resolution.conflicts.len(), 1);
        assert_eq!(resolution.conflicts[0].asset_a, "a");
        assert_eq!(resolution.conflicts[0].asset_b, "b");
    }

    #[test]
    fn cycle_detected() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "a",
            AssetDeps::new().requires(vec![AssetDependency::new("b")]),
        );
        builder.add_asset(
            "b",
            AssetDeps::new().requires(vec![AssetDependency::new("c")]),
        );
        builder.add_asset(
            "c",
            AssetDeps::new().requires(vec![AssetDependency::new("a")]),
        );

        let resolution = builder.resolve(vec!["a".to_string()]);
        assert!(!resolution.graph.cycles.is_empty());
        assert!(resolution.warnings.iter().any(|w| w.contains("cycle")));
    }

    #[test]
    fn unknown_root_reports_error() {
        let builder = DepGraphBuilder::new();
        let resolution = builder.resolve(vec!["nonexistent".to_string()]);
        assert!(!resolution.errors.is_empty());
        assert!(resolution.errors.iter().any(|e| e.contains("nonexistent")));
    }

    #[test]
    fn recommends_reported_as_warning_when_missing() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "app",
            AssetDeps::new().recommends(vec![AssetDependency::new("optional-tool")]),
        );
        builder.add_asset("optional-tool", AssetDeps::new());

        let resolution = builder.resolve(vec!["app".to_string()]);
        assert!(!resolution.warnings.is_empty());
        assert!(resolution
            .warnings
            .iter()
            .any(|w| w.contains("recommends") && w.contains("optional-tool")));
    }

    #[test]
    fn recommends_silent_when_installed() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "app",
            AssetDeps::new()
                .requires(vec![AssetDependency::new("core")])
                .recommends(vec![AssetDependency::new("extra")]),
        );
        builder.add_asset("core", AssetDeps::new());
        builder.add_asset("extra", AssetDeps::new());

        let resolution = builder.resolve(vec!["app".to_string(), "extra".to_string()]);
        let rec_warnings: Vec<&str> = resolution
            .warnings
            .iter()
            .filter(|w| w.contains("recommends"))
            .map(|s| s.as_str())
            .collect();
        assert!(rec_warnings.is_empty(), "got warnings: {rec_warnings:?}");
    }

    #[test]
    fn provides_is_resolved_in_graph() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "tool",
            AssetDeps::new().provides(vec!["capability/x".to_string()]),
        );
        let resolution = builder.resolve(vec!["tool".to_string()]);
        assert_eq!(resolution.resolution.len(), 1);
        assert_eq!(
            resolution.resolution[0].provides,
            vec!["capability/x".to_string()]
        );
    }

    #[test]
    fn find_by_provides_works() {
        let deps = builtin_asset_deps();
        let results = find_asset_by_provides(&deps, "ci/platform/github");
        assert!(!results.is_empty());
        assert!(results.contains(&&"template/github/workflows".to_string()));
    }

    #[test]
    fn builtin_deps_are_consistent() {
        let deps = builtin_asset_deps();
        assert!(deps.contains_key("profile/core/bare"));
        assert!(deps.contains_key("profile/rust"));
        assert!(deps.contains_key("profile/python"));
        assert!(deps.contains_key("profile/node"));
        assert!(deps.contains_key("component/ci"));
        assert!(deps.contains_key("component/docker"));
    }

    #[test]
    fn dot_format_generates_valid_graph() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset(
            "app",
            AssetDeps::new()
                .requires(vec![AssetDependency::new("lib")])
                .conflicts(vec![AssetDependency::new("legacy")]),
        );
        builder.add_asset("lib", AssetDeps::new());
        builder.add_asset("legacy", AssetDeps::new());

        let resolution = builder.resolve(vec!["app".to_string(), "legacy".to_string()]);
        let dot = format_dep_graph_dot(&resolution.graph);

        assert!(dot.starts_with("digraph lode_deps"));
        assert!(dot.contains("\"app\""));
        assert!(dot.contains("\"lib\""));
        assert!(dot.contains("conflict"));
    }

    #[test]
    fn resolution_table_format_includes_order() {
        let mut builder = DepGraphBuilder::new();
        builder.add_asset("root", AssetDeps::new());
        let resolution = builder.resolve(vec!["root".to_string()]);
        let table = format_dep_resolution_table(&resolution);
        assert!(table.contains("Resolution Order"));
        assert!(table.contains("root"));
    }

    #[test]
    fn complex_diamond_dependency() {
        let mut builder = DepGraphBuilder::new();
        // app -> lib-a, lib-b -> common
        builder.add_asset(
            "app",
            AssetDeps::new().requires(vec![
                AssetDependency::new("lib-a"),
                AssetDependency::new("lib-b"),
            ]),
        );
        builder.add_asset(
            "lib-a",
            AssetDeps::new().requires(vec![AssetDependency::new("common")]),
        );
        builder.add_asset(
            "lib-b",
            AssetDeps::new().requires(vec![AssetDependency::new("common")]),
        );
        builder.add_asset("common", AssetDeps::new());

        let resolution = builder.resolve(vec!["app".to_string()]);
        assert_eq!(resolution.resolution.len(), 4);
        assert!(resolution.errors.is_empty());
        assert!(resolution.conflicts.is_empty());

        let depths: HashMap<&str, usize> = resolution
            .resolution
            .iter()
            .map(|r| (r.id.as_str(), r.depth))
            .collect();
        assert_eq!(*depths.get("common").unwrap(), 0);
        assert_eq!(*depths.get("lib-a").unwrap(), 1);
        assert_eq!(*depths.get("lib-b").unwrap(), 1);
        assert_eq!(*depths.get("app").unwrap(), 2);
    }
}
