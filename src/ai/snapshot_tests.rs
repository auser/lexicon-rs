#[cfg(test)]
mod tests {
    use crate::ai::context::assemble_context;
    use chrono::DateTime;
    use crate::spec::common::{RepoType, Severity};
    use crate::spec::contract::{Contract, Invariant, Semantic};
    use crate::spec::gates::GatesModel;
    use crate::spec::manifest::Manifest;
    use crate::spec::scoring::ScoreModel;

    #[test]
    fn snapshot_assemble_context() {
        let fixed = DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
            .unwrap()
            .to_utc();

        let mut manifest = Manifest::new(
            "my-lib".to_string(),
            "A library for testing".to_string(),
            RepoType::Library,
            "key-value store".to_string(),
        );
        manifest.project.created_at = fixed;
        manifest.project.updated_at = fixed;

        let mut contract = Contract::new_draft(
            "kv-store".to_string(),
            "KV Store".to_string(),
            "Basic key-value operations".to_string(),
        );
        contract.created_at = fixed;
        contract.updated_at = fixed;
        contract.invariants.push(Invariant {
            id: "inv-001".to_string(),
            description: "get after set returns the value".to_string(),
            severity: Severity::Required,
            test_tags: vec!["conformance".to_string()],
        });
        contract.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "get returns None for missing keys".to_string(),
            test_tags: vec!["conformance".to_string()],
        });
        contract.forbidden_semantics.push(Semantic {
            id: "forbid-001".to_string(),
            description: "must not panic on missing key".to_string(),
            test_tags: vec!["safety".to_string()],
        });

        let score_model = ScoreModel::default_model();
        let gates_model = GatesModel::default_model();

        let ctx = assemble_context(
            &manifest,
            &[contract],
            Some(&score_model),
            Some(&gates_model),
        );
        insta::assert_snapshot!("assemble_context_full", ctx);
    }
}
