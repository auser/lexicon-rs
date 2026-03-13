#[cfg(test)]
mod tests {
    use crate::common::{RepoType, Severity};
    use crate::contract::{Contract, Invariant, Semantic};
    use crate::gates::GatesModel;
    use crate::manifest::Manifest;
    use crate::scoring::ScoreModel;
    use chrono::DateTime;

    /// Helper: build a sample contract with invariants and semantics.
    fn sample_contract() -> Contract {
        let fixed = DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
            .unwrap()
            .to_utc();
        let mut c = Contract::new_draft(
            "key-value-store".to_string(),
            "Key-Value Store Contract".to_string(),
            "Defines the behavior of a basic key-value store".to_string(),
        );
        c.created_at = fixed;
        c.updated_at = fixed;
        c.capabilities
            .push("get/set/delete operations".to_string());
        c.invariants.push(Invariant {
            id: "inv-001".to_string(),
            description: "A key set with a value must return that value on get".to_string(),
            severity: Severity::Required,
        });
        c.required_semantics.push(Semantic {
            id: "req-001".to_string(),
            description: "get(key) returns None for missing keys".to_string(),
            test_tags: vec!["conformance".to_string(), "basic".to_string()],
        });
        c.forbidden_semantics.push(Semantic {
            id: "forbid-001".to_string(),
            description: "Must not panic on missing key lookup".to_string(),
            test_tags: vec!["safety".to_string()],
        });
        c
    }

    /// Helper: build a sample manifest with fixed timestamps.
    fn sample_manifest() -> Manifest {
        let fixed = DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
            .unwrap()
            .to_utc();
        let mut m = Manifest::new(
            "my-awesome-lib".to_string(),
            "A library for doing awesome things".to_string(),
            RepoType::Library,
            "key-value store".to_string(),
        );
        m.project.created_at = fixed;
        m.project.updated_at = fixed;
        m
    }

    #[test]
    fn snapshot_contract_toml() {
        let contract = sample_contract();
        let toml_str = toml::to_string_pretty(&contract).unwrap();
        insta::assert_snapshot!("contract_toml", toml_str);
    }

    #[test]
    fn snapshot_score_model_toml() {
        let model = ScoreModel::default_model();
        let toml_str = toml::to_string_pretty(&model).unwrap();
        insta::assert_snapshot!("score_model_toml", toml_str);
    }

    #[test]
    fn snapshot_gates_model_toml() {
        let model = GatesModel::default_model();
        let toml_str = toml::to_string_pretty(&model).unwrap();
        insta::assert_snapshot!("gates_model_toml", toml_str);
    }

    #[test]
    fn snapshot_manifest_toml() {
        let manifest = sample_manifest();
        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        insta::assert_snapshot!("manifest_toml", toml_str);
    }
}
