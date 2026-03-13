use lexicon_spec::contract::Contract;
use lexicon_spec::gates::GatesModel;
use lexicon_spec::manifest::Manifest;
use lexicon_spec::scoring::ScoreModel;

/// Assemble AI-readable context from repository state.
///
/// This creates a structured text summary that can be included
/// in prompts or in the managed CLAUDE.md blocks.
pub fn assemble_context(
    manifest: &Manifest,
    contracts: &[Contract],
    score_model: Option<&ScoreModel>,
    gates_model: Option<&GatesModel>,
) -> String {
    let mut lines = Vec::new();

    lines.push(format!("# Project: {}", manifest.project.name));
    lines.push(format!("Domain: {}", manifest.project.domain));
    lines.push(format!("Type: {:?}", manifest.project.repo_type));
    lines.push(String::new());

    if !contracts.is_empty() {
        lines.push("## Contracts".to_string());
        for contract in contracts {
            lines.push(format!(
                "- **{}** ({}): {} [status: {:?}, stability: {:?}]",
                contract.title,
                contract.id,
                contract.scope,
                contract.status,
                contract.stability
            ));

            if !contract.invariants.is_empty() {
                lines.push("  Invariants:".to_string());
                for inv in &contract.invariants {
                    lines.push(format!("  - {}: {}", inv.id, inv.description));
                }
            }

            if !contract.required_semantics.is_empty() {
                lines.push("  Required semantics:".to_string());
                for sem in &contract.required_semantics {
                    lines.push(format!("  - {}: {}", sem.id, sem.description));
                }
            }

            if !contract.forbidden_semantics.is_empty() {
                lines.push("  Forbidden:".to_string());
                for sem in &contract.forbidden_semantics {
                    lines.push(format!("  - {}: {}", sem.id, sem.description));
                }
            }
        }
        lines.push(String::new());
    }

    if let Some(score) = score_model {
        lines.push("## Scoring".to_string());
        lines.push(format!(
            "Pass threshold: {:.0}%, Warn threshold: {:.0}%",
            score.thresholds.pass * 100.0,
            score.thresholds.warn * 100.0
        ));
        for dim in &score.dimensions {
            lines.push(format!(
                "- {} (weight: {}, {:?})",
                dim.label, dim.weight, dim.category
            ));
        }
        lines.push(String::new());
    }

    if let Some(gates) = gates_model {
        lines.push("## Gates".to_string());
        for gate in &gates.gates {
            let skip = if gate.allow_skip { ", skippable" } else { "" };
            lines.push(format!(
                "- {} ({:?}{}): `{}`",
                gate.label, gate.category, skip, gate.command
            ));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::common::RepoType;

    #[test]
    fn test_assemble_context() {
        let manifest = Manifest::new(
            "my-lib".to_string(),
            "A library".to_string(),
            RepoType::Library,
            "key-value store".to_string(),
        );
        let contracts = vec![Contract::new_draft(
            "kv".to_string(),
            "KV Store".to_string(),
            "Basic KV".to_string(),
        )];
        let score = ScoreModel::default_model();
        let gates = GatesModel::default_model();

        let ctx = assemble_context(&manifest, &contracts, Some(&score), Some(&gates));
        assert!(ctx.contains("my-lib"));
        assert!(ctx.contains("KV Store"));
        assert!(ctx.contains("Scoring"));
        assert!(ctx.contains("Gates"));
    }
}
