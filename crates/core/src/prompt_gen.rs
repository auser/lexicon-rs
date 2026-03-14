//! Prompt generation engine — orchestrates artifact collection, template rendering,
//! DAG updates, and staleness detection for implementation prompts.

use std::collections::BTreeMap;
use std::path::Path;

use lexicon_ai::generate::GenerateResult;
use lexicon_ai::generate::GeneratedArtifact;
use lexicon_ai::prompt::ArtifactKind;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::contract::Contract;
use lexicon_spec::gates::GatesModel;
use lexicon_spec::prompt::{
    GraphEdge, GraphNode, NodeKind, PromptMetadata, parse_prompt_file, render_prompt_file,
};
use lexicon_spec::scoring::ScoreModel;

use crate::error::{CoreError, CoreResult};
use crate::prompt_graph;

/// Context gathered from repository artifacts for prompt generation.
pub struct PromptContext {
    pub contract: Contract,
    pub conformance_files: Vec<(String, String)>, // (rel_path, content)
    pub gates_model: Option<GatesModel>,
    pub score_model: Option<ScoreModel>,
    pub architecture_rules: Option<String>,
    pub source_paths: Vec<String>, // all artifact file paths used
}

/// Status of a prompt in the DAG.
pub struct PromptStatus {
    pub node_id: String,
    pub filename: String,
    pub is_stale: bool,
    pub reasons: Vec<String>,
}

/// Explanation of a prompt's dependency chain.
pub struct PromptExplanation {
    pub node_id: String,
    pub path: String,
    pub is_stale: bool,
    pub dependencies: Vec<DependencyDetail>,
}

/// Details about one dependency of a prompt.
pub struct DependencyDetail {
    pub source_id: String,
    pub source_path: String,
    pub stored_hash: String,
    pub current_hash: String,
    pub changed: bool,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate an implementation prompt from a contract.
pub fn generate_prompt(
    layout: &RepoLayout,
    contract_id: &str,
    target: Option<&str>,
    use_ai: bool,
) -> CoreResult<GenerateResult> {
    let ctx = collect_prompt_context(layout, contract_id)?;
    let mut body = render_prompt_body(&ctx, target);

    if use_ai {
        if let Ok(enhanced) = enhance_with_ai(layout, &body) {
            body = enhanced;
        }
    }

    let slug = build_slug(contract_id, target);
    let node_id = format!("prompt:{slug}");
    // Reuse existing prompt number if a prompt for this slug already exists,
    // otherwise allocate the next sequential number.
    let prompt_path = find_existing_prompt(&layout.prompts_dir(), &slug)
        .unwrap_or_else(|| {
            let num = next_prompt_number(&layout.prompts_dir());
            format!("specs/prompts/{num:03}-{slug}.md")
        });

    let (edges, meta, prompt_node) =
        build_graph_updates(layout, &ctx, &node_id, &prompt_path);

    let content = render_prompt_file(&meta, &body);

    // Update the DAG
    let mut graph = prompt_graph::load_graph(layout)?;
    // Upsert all source nodes with current hashes
    let source_nodes = prompt_graph::discover_source_nodes(layout)?;
    for sn in source_nodes {
        prompt_graph::upsert_node(&mut graph, sn);
    }
    prompt_graph::upsert_node(&mut graph, prompt_node);
    prompt_graph::add_edges(&mut graph, edges);
    graph.built_at = chrono::Utc::now().to_rfc3339();
    prompt_graph::save_graph(layout, &graph)?;

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::ImplementationPrompt,
            path: prompt_path,
            content,
            format: "markdown".to_string(),
        },
        warnings: Vec::new(),
    })
}

/// Regenerate all stale prompts.
pub fn regenerate_stale(
    layout: &RepoLayout,
    use_ai: bool,
) -> CoreResult<Vec<GenerateResult>> {
    let graph = prompt_graph::load_graph(layout)?;
    let dirty = prompt_graph::find_dirty_sources(layout, &graph)?;
    if dirty.is_empty() {
        return Ok(Vec::new());
    }

    let affected = prompt_graph::find_affected_prompts(&graph, &dirty);
    let mut results = Vec::new();

    for node_id in &affected {
        if let Some(node) = graph.nodes.iter().find(|n| &n.id == node_id) {
            let full_path = layout.root.join(&node.path);
            if let Ok(result) = regenerate_from_file(layout, &full_path, use_ai) {
                results.push(result);
            }
        }
    }

    Ok(results)
}

/// Regenerate a specific prompt by filename (e.g. "001-memory-blob-store").
pub fn regenerate_one(
    layout: &RepoLayout,
    prompt_name: &str,
    use_ai: bool,
) -> CoreResult<GenerateResult> {
    let prompts_dir = layout.prompts_dir();
    let filename = if prompt_name.ends_with(".md") {
        prompt_name.to_string()
    } else {
        format!("{prompt_name}.md")
    };
    let full_path = prompts_dir.join(&filename);
    if !full_path.exists() {
        return Err(CoreError::Other(format!(
            "Prompt file not found: {filename}"
        )));
    }
    regenerate_from_file(layout, &full_path, use_ai)
}

/// Check staleness status of all prompts.
pub fn check_all_prompt_statuses(layout: &RepoLayout) -> CoreResult<Vec<PromptStatus>> {
    let graph = prompt_graph::load_graph(layout)?;
    let dirty = prompt_graph::find_dirty_sources(layout, &graph)?;
    let affected = prompt_graph::find_affected_prompts(&graph, &dirty);

    let mut statuses = Vec::new();
    for node in &graph.nodes {
        if node.kind != NodeKind::Prompt {
            continue;
        }
        let is_stale = affected.contains(&node.id);
        let reasons = if is_stale {
            // Find which dirty sources affect this prompt
            graph
                .edges
                .iter()
                .filter(|e| e.to == node.id && dirty.contains(&e.from))
                .map(|e| {
                    let source_node = graph.nodes.iter().find(|n| n.id == e.from);
                    match source_node {
                        Some(sn) => format!("{} changed", sn.path),
                        None => format!("{} changed", e.from),
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let filename = Path::new(&node.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        statuses.push(PromptStatus {
            node_id: node.id.clone(),
            filename,
            is_stale,
            reasons,
        });
    }

    // Also check for prompt files that exist but aren't in the graph
    let prompts_dir = layout.prompts_dir();
    if prompts_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&prompts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    // Check if it has frontmatter and is tracked
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if parse_prompt_file(&content).is_some()
                            && !statuses.iter().any(|s| s.filename == filename)
                        {
                            statuses.push(PromptStatus {
                                node_id: String::new(),
                                filename,
                                is_stale: true,
                                reasons: vec!["not tracked in prompt graph".to_string()],
                            });
                        }
                    }
                }
            }
        }
    }

    statuses.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(statuses)
}

/// Explain the dependency chain of a prompt.
pub fn explain_prompt(
    layout: &RepoLayout,
    prompt_name: &str,
) -> CoreResult<PromptExplanation> {
    let prompts_dir = layout.prompts_dir();
    let filename = if prompt_name.ends_with(".md") {
        prompt_name.to_string()
    } else {
        format!("{prompt_name}.md")
    };
    let full_path = prompts_dir.join(&filename);
    if !full_path.exists() {
        return Err(CoreError::Other(format!(
            "Prompt file not found: {filename}"
        )));
    }

    let content = std::fs::read_to_string(&full_path)?;
    let (meta, _body) = parse_prompt_file(&content).ok_or_else(|| {
        CoreError::Other("Prompt file has no valid frontmatter metadata".to_string())
    })?;

    let mut dependencies = Vec::new();
    let mut is_stale = false;

    for (artifact_path, stored_hash) in &meta.artifact_hashes {
        let abs_path = layout.root.join(artifact_path);
        let current_hash = prompt_graph::hash_file(&abs_path).unwrap_or_default();
        let changed = current_hash != *stored_hash;
        if changed {
            is_stale = true;
        }
        // Find the source ID from depends_on that corresponds to this path
        let source_id = meta
            .depends_on
            .iter()
            .zip(meta.artifact_paths.iter())
            .find(|(_, p)| *p == artifact_path)
            .map(|(id, _)| id.clone())
            .unwrap_or_default();

        dependencies.push(DependencyDetail {
            source_id,
            source_path: artifact_path.clone(),
            stored_hash: stored_hash.clone(),
            current_hash,
            changed,
        });
    }

    Ok(PromptExplanation {
        node_id: meta.node_id,
        path: filename,
        is_stale,
        dependencies,
    })
}

/// List all prompt files in specs/prompts/ (sorted).
pub fn list_prompts(layout: &RepoLayout) -> CoreResult<Vec<String>> {
    let prompts_dir = layout.prompts_dir();
    let mut files = Vec::new();
    if prompts_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&prompts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(name) = path.file_name() {
                        files.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

pub(crate) fn collect_prompt_context(
    layout: &RepoLayout,
    contract_id: &str,
) -> CoreResult<PromptContext> {
    let mut source_paths = Vec::new();

    // Load contract
    let contract_path = layout.contracts_dir().join(format!("{contract_id}.toml"));
    let contract_text = std::fs::read_to_string(&contract_path).map_err(|_| {
        CoreError::Other(format!("Contract not found: {contract_id}"))
    })?;
    let contract: Contract = toml::from_str(&contract_text)
        .map_err(|e| CoreError::Other(format!("Failed to parse contract: {e}")))?;
    source_paths.push(path_rel(&contract_path, &layout.root));

    // Scan conformance tests for references to this contract's test tags
    let mut conformance_files = Vec::new();
    let conformance_dir = layout.conformance_tests_dir();
    if conformance_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&conformance_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains(&contract.id) || has_matching_tags(&content, &contract) {
                            let rel = path_rel(&path, &layout.root);
                            source_paths.push(rel.clone());
                            conformance_files.push((rel, content));
                        }
                    }
                }
            }
        }
    }

    // Gates
    let gates_path = layout.gates_path();
    let gates_model = if gates_path.is_file() {
        source_paths.push(path_rel(&gates_path, &layout.root));
        std::fs::read_to_string(&gates_path)
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
    } else {
        None
    };

    // Scoring
    let scoring_path = layout.scoring_model_path();
    let score_model = if scoring_path.is_file() {
        source_paths.push(path_rel(&scoring_path, &layout.root));
        std::fs::read_to_string(&scoring_path)
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
    } else {
        None
    };

    // Architecture rules
    let arch_path = layout.architecture_rules_path();
    let architecture_rules = if arch_path.is_file() {
        source_paths.push(path_rel(&arch_path, &layout.root));
        std::fs::read_to_string(&arch_path).ok()
    } else {
        None
    };

    Ok(PromptContext {
        contract,
        conformance_files,
        gates_model,
        score_model,
        architecture_rules,
        source_paths,
    })
}

pub(crate) fn render_prompt_body(ctx: &PromptContext, target: Option<&str>) -> String {
    let mut body = String::new();
    let c = &ctx.contract;

    // Title
    body.push_str(&format!("# IMPLEMENTATION PROMPT — {}\n\n", c.title));

    // Objective
    body.push_str("## Objective\n\n");
    body.push_str(&format!(
        "Implement code that satisfies the **{}** contract (`{}`).\n",
        c.title, c.id
    ));
    if let Some(t) = target {
        body.push_str(&format!("Target implementation variant: **{t}**.\n"));
    }
    body.push_str(&format!("\n**Scope:** {}\n\n", c.scope));

    // Repository Context
    body.push_str("## Repository Context\n\n");
    body.push_str("This prompt was generated by Lexicon from repository artifacts.\n");
    body.push_str("The implementation must satisfy the contract-defined behavior.\n\n");

    // Existing Artifacts
    body.push_str("## Existing Artifacts\n\n");
    body.push_str(&format!(
        "- **Contract:** `specs/contracts/{}.toml`\n",
        c.id
    ));
    for (path, _) in &ctx.conformance_files {
        body.push_str(&format!("- **Conformance test:** `{path}`\n"));
    }
    if ctx.gates_model.is_some() {
        body.push_str("- **Gates:** `specs/gates.toml`\n");
    }
    if ctx.score_model.is_some() {
        body.push_str("- **Scoring:** `specs/scoring/model.toml`\n");
    }
    if ctx.architecture_rules.is_some() {
        body.push_str("- **Architecture:** `.lexicon/architecture/rules.toml`\n");
    }
    body.push('\n');

    // Behavioral Requirements
    body.push_str("## Behavioral Requirements\n\n");

    if !c.invariants.is_empty() {
        body.push_str("### Invariants\n\n");
        for inv in &c.invariants {
            body.push_str(&format!(
                "- **{}**: {} (severity: {:?})\n",
                inv.id, inv.description, inv.severity
            ));
        }
        body.push('\n');
    }

    if !c.required_semantics.is_empty() {
        body.push_str("### Required Semantics\n\n");
        for sem in &c.required_semantics {
            body.push_str(&format!("- **{}**: {}\n", sem.id, sem.description));
        }
        body.push('\n');
    }

    if !c.forbidden_semantics.is_empty() {
        body.push_str("### Forbidden Semantics\n\n");
        for sem in &c.forbidden_semantics {
            body.push_str(&format!("- **{}**: {}\n", sem.id, sem.description));
        }
        body.push('\n');
    }

    if !c.edge_cases.is_empty() {
        body.push_str("### Edge Cases\n\n");
        for ec in &c.edge_cases {
            body.push_str(&format!(
                "- **{}**: {} → {}\n",
                ec.id, ec.scenario, ec.expected_behavior
            ));
        }
        body.push('\n');
    }

    // Files To Create Or Modify
    body.push_str("## Files To Create Or Modify\n\n");
    if let Some(t) = target {
        body.push_str(&format!(
            "- `src/{}.rs` — {t} implementation of `{}`\n",
            t, c.id
        ));
    } else {
        body.push_str(&format!(
            "- Implementation file(s) for `{}`\n",
            c.id
        ));
    }
    body.push('\n');

    // Constraints
    body.push_str("## Constraints\n\n");
    body.push_str("- Do not modify existing contracts\n");
    body.push_str("- Do not weaken existing conformance tests\n");
    body.push_str("- Do not bypass verification gates\n");
    if let Some(ref rules) = ctx.architecture_rules {
        body.push_str("- Respect architecture rules:\n");
        for line in rules.lines().take(10) {
            body.push_str(&format!("  {line}\n"));
        }
    }
    body.push('\n');

    // Verification Requirements
    body.push_str("## Verification Requirements\n\n");
    if !ctx.conformance_files.is_empty() {
        body.push_str("The following conformance tests must pass:\n\n");
        for (path, _) in &ctx.conformance_files {
            body.push_str(&format!("- `{path}`\n"));
        }
        body.push('\n');
    }
    if let Some(ref gates) = ctx.gates_model {
        body.push_str("The following gates must pass:\n\n");
        for gate in &gates.gates {
            body.push_str(&format!("- **{}**: `{}`\n", gate.id, gate.command));
        }
        body.push('\n');
    }

    // Acceptance Criteria
    body.push_str("## Acceptance Criteria\n\n");
    for inv in &c.invariants {
        body.push_str(&format!("- [ ] Invariant `{}` holds: {}\n", inv.id, inv.description));
    }
    for sem in &c.required_semantics {
        body.push_str(&format!("- [ ] Required `{}`: {}\n", sem.id, sem.description));
    }
    for sem in &c.forbidden_semantics {
        body.push_str(&format!(
            "- [ ] Forbidden `{}` is prevented: {}\n",
            sem.id, sem.description
        ));
    }
    body.push('\n');

    // Artifact Dependencies
    body.push_str("## Artifact Dependencies\n\n");
    body.push_str("This prompt was derived from the following artifacts:\n\n");
    for path in &ctx.source_paths {
        body.push_str(&format!("- `{path}`\n"));
    }
    body.push('\n');

    body
}

pub(crate) fn build_graph_updates(
    layout: &RepoLayout,
    ctx: &PromptContext,
    prompt_node_id: &str,
    prompt_path: &str,
) -> (Vec<GraphEdge>, PromptMetadata, GraphNode) {
    let now = chrono::Utc::now().to_rfc3339();

    // Hash all source files
    let mut artifact_hashes = BTreeMap::new();
    let mut depends_on = Vec::new();

    for source_path in &ctx.source_paths {
        let abs = layout.root.join(source_path);
        let hash = prompt_graph::hash_file(&abs).unwrap_or_default();
        artifact_hashes.insert(source_path.clone(), hash);

        // Derive node ID from path
        let node_id = source_path_to_node_id(source_path);
        if !depends_on.contains(&node_id) {
            depends_on.push(node_id);
        }
    }

    let edges: Vec<GraphEdge> = depends_on
        .iter()
        .map(|src_id| GraphEdge {
            from: src_id.clone(),
            to: prompt_node_id.to_string(),
        })
        .collect();

    let meta = PromptMetadata {
        prompt_version: 1,
        generated_at: now.clone(),
        node_id: prompt_node_id.to_string(),
        depends_on,
        artifact_paths: ctx.source_paths.clone(),
        artifact_hashes,
    };

    let prompt_node = GraphNode {
        id: prompt_node_id.to_string(),
        kind: NodeKind::Prompt,
        path: prompt_path.to_string(),
        hash: String::new(), // Will be set after content is written
        updated_at: now,
    };

    (edges, meta, prompt_node)
}

fn enhance_with_ai(layout: &RepoLayout, raw_body: &str) -> CoreResult<String> {
    let provider = crate::generate::build_ai_provider(layout, None)?;
    let system = lexicon_ai::prompt::system_prompt(ArtifactKind::ImplementationPrompt);
    let result = provider
        .complete(system, raw_body)
        .map_err(|e| CoreError::Other(format!("AI enhancement failed: {e}")))?;
    Ok(result)
}

fn regenerate_from_file(
    layout: &RepoLayout,
    prompt_path: &Path,
    use_ai: bool,
) -> CoreResult<GenerateResult> {
    let content = std::fs::read_to_string(prompt_path)?;
    let (meta, _body) = parse_prompt_file(&content).ok_or_else(|| {
        CoreError::Other("Cannot regenerate: no valid frontmatter".to_string())
    })?;

    // Extract contract_id from node_id "prompt:<slug>"
    let slug = meta.node_id.strip_prefix("prompt:").unwrap_or(&meta.node_id);
    // Try to find the contract_id from depends_on
    let contract_id = meta
        .depends_on
        .iter()
        .find(|d| d.starts_with("contract:"))
        .and_then(|d| d.strip_prefix("contract:"))
        .unwrap_or(slug);

    let ctx = collect_prompt_context(layout, contract_id)?;
    let mut body = render_prompt_body(&ctx, None);

    if use_ai {
        if let Ok(enhanced) = enhance_with_ai(layout, &body) {
            body = enhanced;
        }
    }

    let filename = prompt_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let rel_path = format!("specs/prompts/{filename}");

    let (edges, mut new_meta, prompt_node) =
        build_graph_updates(layout, &ctx, &meta.node_id, &rel_path);
    new_meta.prompt_version = meta.prompt_version + 1;

    let new_content = render_prompt_file(&new_meta, &body);

    // Update graph
    let mut graph = prompt_graph::load_graph(layout)?;
    let source_nodes = prompt_graph::discover_source_nodes(layout)?;
    for sn in source_nodes {
        prompt_graph::upsert_node(&mut graph, sn);
    }
    prompt_graph::upsert_node(&mut graph, prompt_node);
    prompt_graph::add_edges(&mut graph, edges);
    graph.built_at = chrono::Utc::now().to_rfc3339();
    prompt_graph::save_graph(layout, &graph)?;

    Ok(GenerateResult {
        artifact: GeneratedArtifact {
            kind: ArtifactKind::ImplementationPrompt,
            path: rel_path,
            content: new_content,
            format: "markdown".to_string(),
        },
        warnings: Vec::new(),
    })
}

/// Find an existing prompt file matching a slug (e.g., `001-kv-store.md`).
/// Returns the relative path like `specs/prompts/001-kv-store.md` if found.
fn find_existing_prompt(prompts_dir: &Path, slug: &str) -> Option<String> {
    let suffix = format!("-{slug}.md");
    if let Ok(entries) = std::fs::read_dir(prompts_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(&suffix) {
                return Some(format!("specs/prompts/{name}"));
            }
        }
    }
    None
}

pub(crate) fn next_prompt_number(prompts_dir: &Path) -> u32 {
    let mut max = 0u32;
    if let Ok(entries) = std::fs::read_dir(prompts_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".md") {
                if let Some(num_str) = name.get(..3) {
                    if let Ok(num) = num_str.parse::<u32>() {
                        max = max.max(num);
                    }
                }
            }
        }
    }
    max + 1
}

pub(crate) fn build_slug(contract_id: &str, target: Option<&str>) -> String {
    match target {
        Some(t) => format!("{contract_id}-{t}"),
        None => contract_id.to_string(),
    }
}

fn has_matching_tags(content: &str, contract: &Contract) -> bool {
    for inv in &contract.invariants {
        for tag in &inv.test_tags {
            if content.contains(tag) {
                return true;
            }
        }
    }
    for sem in &contract.required_semantics {
        for tag in &sem.test_tags {
            if content.contains(tag) {
                return true;
            }
        }
    }
    false
}

/// Derive a graph node ID from a relative source file path.
fn source_path_to_node_id(path: &str) -> String {
    if path.starts_with("specs/contracts/") {
        let stem = Path::new(path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        format!("contract:{stem}")
    } else if path.starts_with("tests/conformance/") {
        let stem = Path::new(path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        format!("conformance:{stem}")
    } else if path.contains("gates") {
        "gate:gates".to_string()
    } else if path.contains("scoring") {
        "scoring:model".to_string()
    } else if path.contains("architecture") {
        "arch:rules".to_string()
    } else if path.contains("api") {
        "api:baseline".to_string()
    } else {
        format!("source:{}", path.replace('/', "-"))
    }
}

fn path_rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_prompt_number_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(next_prompt_number(dir.path()), 1);
    }

    #[test]
    fn test_next_prompt_number_existing() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(dir.path().join("001-test.md"), "").unwrap();
        std::fs::write(dir.path().join("005-other.md"), "").unwrap();
        assert_eq!(next_prompt_number(dir.path()), 6);
    }

    #[test]
    fn test_build_slug() {
        assert_eq!(build_slug("blob-store", None), "blob-store");
        assert_eq!(
            build_slug("blob-store", Some("memory")),
            "blob-store-memory"
        );
    }

    #[test]
    fn test_source_path_to_node_id() {
        assert_eq!(
            source_path_to_node_id("specs/contracts/blob-store.toml"),
            "contract:blob-store"
        );
        assert_eq!(
            source_path_to_node_id("tests/conformance/blob-store.rs"),
            "conformance:blob-store"
        );
        assert_eq!(
            source_path_to_node_id("specs/gates.toml"),
            "gate:gates"
        );
        assert_eq!(
            source_path_to_node_id("specs/scoring/model.toml"),
            "scoring:model"
        );
    }

    #[test]
    fn test_list_prompts() {
        let dir = tempfile::TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        let prompts_dir = layout.prompts_dir();
        std::fs::create_dir_all(&prompts_dir).unwrap();
        std::fs::write(prompts_dir.join("002-b.md"), "").unwrap();
        std::fs::write(prompts_dir.join("001-a.md"), "").unwrap();

        let list = list_prompts(&layout).unwrap();
        assert_eq!(list, vec!["001-a.md", "002-b.md"]);
    }
}
