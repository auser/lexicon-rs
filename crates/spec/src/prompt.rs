//! Prompt DAG types — dependency graph and metadata for implementation prompts.
//!
//! Implementation prompts are derived artifacts compiled from repository law
//! (contracts, conformance tests, gates, etc.). This module defines the graph
//! structure that tracks dependencies and enables incremental rebuilds.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Node types in the prompt dependency DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Contract,
    Conformance,
    Gate,
    ScoringModel,
    ApiBaseline,
    ArchitectureRule,
    Prompt,
}

/// A node in the prompt DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Stable identifier, e.g. "contract:blob-store" or "prompt:memory-blob-store".
    pub id: String,
    /// What kind of artifact this node represents.
    pub kind: NodeKind,
    /// File path relative to repo root.
    pub path: String,
    /// SHA-256 hex digest of the file contents at last build.
    pub hash: String,
    /// ISO 8601 timestamp of last update.
    pub updated_at: String,
}

/// A directed edge from a source artifact to a derived prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID (e.g. "contract:blob-store").
    pub from: String,
    /// Derived node ID (e.g. "prompt:memory-blob-store").
    pub to: String,
}

/// The full prompt dependency graph, persisted at `.lexicon/prompt-graph.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGraph {
    pub schema_version: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub built_at: String,
}

impl Default for PromptGraph {
    fn default() -> Self {
        Self {
            schema_version: "0.1.0".to_string(),
            nodes: Vec::new(),
            edges: Vec::new(),
            built_at: String::new(),
        }
    }
}

/// YAML frontmatter metadata embedded in each generated prompt file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    /// Incremented on each regeneration.
    pub prompt_version: u32,
    /// ISO 8601 timestamp of generation.
    pub generated_at: String,
    /// Stable node ID in the DAG, e.g. "prompt:memory-blob-store".
    pub node_id: String,
    /// Source node IDs this prompt depends on.
    pub depends_on: Vec<String>,
    /// Source file paths (relative to repo root).
    pub artifact_paths: Vec<String>,
    /// Map of source file path to SHA-256 hex digest at generation time.
    pub artifact_hashes: BTreeMap<String, String>,
}

/// Parse a prompt file into its metadata frontmatter and body content.
///
/// The file format is:
/// ```text
/// ---
/// <yaml frontmatter>
/// ---
/// <markdown body>
/// ```
///
/// Returns `None` if the file does not contain valid frontmatter.
pub fn parse_prompt_file(content: &str) -> Option<(PromptMetadata, String)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    // Find the second "---" delimiter
    let after_first = &content[3..];
    let end_idx = after_first.find("\n---")?;
    let yaml_block = &after_first[..end_idx].trim();
    let body_start = 3 + end_idx + 4; // skip past "\n---"
    let body = if body_start < content.len() {
        content[body_start..].trim_start_matches('\n').to_string()
    } else {
        String::new()
    };

    let meta: PromptMetadata = serde_yaml::from_str(yaml_block).ok()?;
    Some((meta, body))
}

/// Render a prompt file with YAML frontmatter metadata and markdown body.
pub fn render_prompt_file(meta: &PromptMetadata, body: &str) -> String {
    let yaml = serde_yaml::to_string(meta).unwrap_or_default();
    format!("---\n{yaml}---\n\n{body}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_graph_default() {
        let graph = PromptGraph::default();
        assert_eq!(graph.schema_version, "0.1.0");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_node_kind_serde() {
        let kind = NodeKind::Contract;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"contract\"");
        let parsed: NodeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(kind, parsed);
    }

    #[test]
    fn test_parse_and_render_prompt_file() {
        let meta = PromptMetadata {
            prompt_version: 1,
            generated_at: "2026-03-13T12:00:00Z".to_string(),
            node_id: "prompt:test-prompt".to_string(),
            depends_on: vec!["contract:test".to_string()],
            artifact_paths: vec!["specs/contracts/test.toml".to_string()],
            artifact_hashes: BTreeMap::from([(
                "specs/contracts/test.toml".to_string(),
                "abc123".to_string(),
            )]),
        };

        let body = "# IMPLEMENTATION PROMPT -- Test\n\n## Objective\n\nTest objective.";
        let rendered = render_prompt_file(&meta, body);

        assert!(rendered.starts_with("---\n"));
        assert!(rendered.contains("prompt_version: 1"));
        assert!(rendered.contains("node_id: 'prompt:test-prompt'") || rendered.contains("node_id: prompt:test-prompt"));
        assert!(rendered.contains("# IMPLEMENTATION PROMPT -- Test"));

        // Round-trip
        let (parsed_meta, parsed_body) = parse_prompt_file(&rendered).unwrap();
        assert_eq!(parsed_meta.prompt_version, 1);
        assert_eq!(parsed_meta.node_id, "prompt:test-prompt");
        assert_eq!(parsed_meta.depends_on, vec!["contract:test"]);
        assert!(parsed_body.contains("# IMPLEMENTATION PROMPT -- Test"));
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a regular markdown file\n\nNo frontmatter here.";
        assert!(parse_prompt_file(content).is_none());
    }

    #[test]
    fn test_prompt_graph_serde_roundtrip() {
        let graph = PromptGraph {
            schema_version: "0.1.0".to_string(),
            nodes: vec![GraphNode {
                id: "contract:test".to_string(),
                kind: NodeKind::Contract,
                path: "specs/contracts/test.toml".to_string(),
                hash: "abc123".to_string(),
                updated_at: "2026-03-13T12:00:00Z".to_string(),
            }],
            edges: vec![GraphEdge {
                from: "contract:test".to_string(),
                to: "prompt:test-impl".to_string(),
            }],
            built_at: "2026-03-13T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string_pretty(&graph).unwrap();
        let parsed: PromptGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.edges.len(), 1);
        assert_eq!(parsed.nodes[0].id, "contract:test");
    }
}
