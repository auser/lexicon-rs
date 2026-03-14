//! Prompt DAG engine — graph operations for dependency tracking and staleness detection.

use std::path::Path;

use sha2::{Digest, Sha256};

use lexicon_repo::layout::RepoLayout;
use lexicon_spec::prompt::{GraphEdge, GraphNode, NodeKind, PromptGraph};

use crate::error::{CoreError, CoreResult};

/// Compute SHA-256 hex hash of a file's contents. Returns `None` if the file is missing.
pub fn hash_file(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let digest = Sha256::digest(&bytes);
    Some(format!("{:x}", digest))
}

/// Load the prompt graph from `.lexicon/prompt-graph.json`, or return an empty default graph.
pub fn load_graph(layout: &RepoLayout) -> CoreResult<PromptGraph> {
    let path = layout.prompt_graph_path();
    if !path.exists() {
        return Ok(PromptGraph::default());
    }
    let text = std::fs::read_to_string(&path)?;
    let graph: PromptGraph = serde_json::from_str(&text)
        .map_err(|e| CoreError::Other(format!("Failed to parse prompt graph: {e}")))?;
    Ok(graph)
}

/// Save the prompt graph to `.lexicon/prompt-graph.json`.
pub fn save_graph(layout: &RepoLayout, graph: &PromptGraph) -> CoreResult<()> {
    let path = layout.prompt_graph_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(graph)
        .map_err(|e| CoreError::Other(format!("Failed to serialize prompt graph: {e}")))?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Discover all source artifact nodes by scanning the repository.
///
/// Scans contracts, conformance tests, gates, scoring, architecture rules, and API baseline.
/// Returns nodes with current file hashes.
pub fn discover_source_nodes(layout: &RepoLayout) -> CoreResult<Vec<GraphNode>> {
    let mut nodes = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    // Contracts: specs/contracts/*.toml
    let contracts_dir = layout.contracts_dir();
    if contracts_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&contracts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    let rel = path_relative_to(&path, &layout.root);
                    if let Some(hash) = hash_file(&path) {
                        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                        nodes.push(GraphNode {
                            id: format!("contract:{stem}"),
                            kind: NodeKind::Contract,
                            path: rel,
                            hash,
                            updated_at: now.clone(),
                        });
                    }
                }
            }
        }
    }

    // Conformance tests: tests/conformance/*.rs
    let conformance_dir = layout.conformance_tests_dir();
    if conformance_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&conformance_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    let rel = path_relative_to(&path, &layout.root);
                    if let Some(hash) = hash_file(&path) {
                        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                        nodes.push(GraphNode {
                            id: format!("conformance:{stem}"),
                            kind: NodeKind::Conformance,
                            path: rel,
                            hash,
                            updated_at: now.clone(),
                        });
                    }
                }
            }
        }
    }

    // Gates: specs/gates.toml
    let gates_path = layout.gates_path();
    if gates_path.is_file() {
        if let Some(hash) = hash_file(&gates_path) {
            nodes.push(GraphNode {
                id: "gate:gates".to_string(),
                kind: NodeKind::Gate,
                path: path_relative_to(&gates_path, &layout.root),
                hash,
                updated_at: now.clone(),
            });
        }
    }

    // Scoring: specs/scoring/model.toml
    let scoring_path = layout.scoring_model_path();
    if scoring_path.is_file() {
        if let Some(hash) = hash_file(&scoring_path) {
            nodes.push(GraphNode {
                id: "scoring:model".to_string(),
                kind: NodeKind::ScoringModel,
                path: path_relative_to(&scoring_path, &layout.root),
                hash,
                updated_at: now.clone(),
            });
        }
    }

    // Architecture rules: .lexicon/architecture/rules.toml
    let arch_path = layout.architecture_rules_path();
    if arch_path.is_file() {
        if let Some(hash) = hash_file(&arch_path) {
            nodes.push(GraphNode {
                id: "arch:rules".to_string(),
                kind: NodeKind::ArchitectureRule,
                path: path_relative_to(&arch_path, &layout.root),
                hash,
                updated_at: now.clone(),
            });
        }
    }

    // API baseline: .lexicon/api/current.json
    let api_path = layout.api_dir().join("current.json");
    if api_path.is_file() {
        if let Some(hash) = hash_file(&api_path) {
            nodes.push(GraphNode {
                id: "api:baseline".to_string(),
                kind: NodeKind::ApiBaseline,
                path: path_relative_to(&api_path, &layout.root),
                hash,
                updated_at: now.clone(),
            });
        }
    }

    Ok(nodes)
}

/// Compare stored node hashes against current file hashes.
/// Returns IDs of source nodes whose files have changed.
pub fn find_dirty_sources(layout: &RepoLayout, graph: &PromptGraph) -> CoreResult<Vec<String>> {
    let mut dirty = Vec::new();
    for node in &graph.nodes {
        if node.kind == NodeKind::Prompt {
            continue;
        }
        let full_path = layout.root.join(&node.path);
        let current_hash = hash_file(&full_path).unwrap_or_default();
        if current_hash != node.hash {
            dirty.push(node.id.clone());
        }
    }
    Ok(dirty)
}

/// Traverse edges from dirty source IDs to find all affected prompt node IDs.
pub fn find_affected_prompts(graph: &PromptGraph, dirty: &[String]) -> Vec<String> {
    let mut affected = Vec::new();
    for edge in &graph.edges {
        if dirty.contains(&edge.from) && !affected.contains(&edge.to) {
            affected.push(edge.to.clone());
        }
    }
    affected
}

/// Add or update a node in the graph by ID.
pub fn upsert_node(graph: &mut PromptGraph, node: GraphNode) {
    if let Some(existing) = graph.nodes.iter_mut().find(|n| n.id == node.id) {
        *existing = node;
    } else {
        graph.nodes.push(node);
    }
}

/// Add edges, skipping duplicates.
pub fn add_edges(graph: &mut PromptGraph, edges: Vec<GraphEdge>) {
    for edge in edges {
        let exists = graph
            .edges
            .iter()
            .any(|e| e.from == edge.from && e.to == edge.to);
        if !exists {
            graph.edges.push(edge);
        }
    }
}

/// Remove a prompt node and all its edges.
pub fn remove_prompt_node(graph: &mut PromptGraph, node_id: &str) {
    graph.nodes.retain(|n| n.id != node_id);
    graph.edges.retain(|e| e.from != node_id && e.to != node_id);
}

/// Get a relative path string from an absolute path and a root.
fn path_relative_to(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_file() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello world").unwrap();
        let hash = hash_file(&file).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_hash_file_missing() {
        assert!(hash_file(Path::new("/nonexistent/file.txt")).is_none());
    }

    #[test]
    fn test_load_save_graph() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        std::fs::create_dir_all(layout.lexicon_dir()).unwrap();

        let mut graph = PromptGraph::default();
        graph.nodes.push(GraphNode {
            id: "contract:test".to_string(),
            kind: NodeKind::Contract,
            path: "specs/contracts/test.toml".to_string(),
            hash: "abc".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        });

        save_graph(&layout, &graph).unwrap();
        let loaded = load_graph(&layout).unwrap();
        assert_eq!(loaded.nodes.len(), 1);
        assert_eq!(loaded.nodes[0].id, "contract:test");
    }

    #[test]
    fn test_load_graph_missing() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        let graph = load_graph(&layout).unwrap();
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_find_dirty_sources() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        // Create a source file
        let contracts_dir = layout.contracts_dir();
        std::fs::create_dir_all(&contracts_dir).unwrap();
        let contract_file = contracts_dir.join("test.toml");
        std::fs::write(&contract_file, "original content").unwrap();

        let hash = hash_file(&contract_file).unwrap();
        let graph = PromptGraph {
            schema_version: "0.1.0".to_string(),
            nodes: vec![GraphNode {
                id: "contract:test".to_string(),
                kind: NodeKind::Contract,
                path: "specs/contracts/test.toml".to_string(),
                hash,
                updated_at: String::new(),
            }],
            edges: Vec::new(),
            built_at: String::new(),
        };

        // Not dirty yet
        let dirty = find_dirty_sources(&layout, &graph).unwrap();
        assert!(dirty.is_empty());

        // Modify the file
        std::fs::write(&contract_file, "modified content").unwrap();
        let dirty = find_dirty_sources(&layout, &graph).unwrap();
        assert_eq!(dirty, vec!["contract:test"]);
    }

    #[test]
    fn test_find_affected_prompts() {
        let graph = PromptGraph {
            schema_version: "0.1.0".to_string(),
            nodes: Vec::new(),
            edges: vec![
                GraphEdge {
                    from: "contract:blob-store".to_string(),
                    to: "prompt:memory-blob-store".to_string(),
                },
                GraphEdge {
                    from: "contract:blob-store".to_string(),
                    to: "prompt:file-blob-store".to_string(),
                },
                GraphEdge {
                    from: "gate:gates".to_string(),
                    to: "prompt:memory-blob-store".to_string(),
                },
            ],
            built_at: String::new(),
        };

        let affected = find_affected_prompts(&graph, &["contract:blob-store".to_string()]);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&"prompt:memory-blob-store".to_string()));
        assert!(affected.contains(&"prompt:file-blob-store".to_string()));
    }

    #[test]
    fn test_upsert_node() {
        let mut graph = PromptGraph::default();
        let node = GraphNode {
            id: "contract:test".to_string(),
            kind: NodeKind::Contract,
            path: "test.toml".to_string(),
            hash: "abc".to_string(),
            updated_at: String::new(),
        };
        upsert_node(&mut graph, node.clone());
        assert_eq!(graph.nodes.len(), 1);

        // Update existing
        let updated = GraphNode {
            hash: "def".to_string(),
            ..node
        };
        upsert_node(&mut graph, updated);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].hash, "def");
    }

    #[test]
    fn test_add_edges_dedup() {
        let mut graph = PromptGraph::default();
        let edge = GraphEdge {
            from: "a".to_string(),
            to: "b".to_string(),
        };
        add_edges(&mut graph, vec![edge.clone()]);
        add_edges(&mut graph, vec![edge]);
        assert_eq!(graph.edges.len(), 1);
    }
}
