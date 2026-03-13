use lexicon_spec::common::ConformanceStyle;
use lexicon_spec::contract::Contract;

use crate::templates;

/// Generate conformance test code for a contract.
pub fn generate_conformance_code(contract: &Contract, style: ConformanceStyle) -> String {
    match style {
        ConformanceStyle::TraitBased => templates::trait_based_harness(contract),
        ConformanceStyle::FactoryBased => templates::factory_based_harness(contract),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::common::Severity;
    use lexicon_spec::contract::{Invariant, Semantic};

    fn sample_contract() -> Contract {
        let mut c = Contract::new_draft(
            "key-value-store".to_string(),
            "Key-Value Store".to_string(),
            "Basic key-value storage operations".to_string(),
        );
        c.invariants.push(Invariant {
            id: "inv-get-after-set".to_string(),
            description: "A key set with a value must return that value on get".to_string(),
            severity: Severity::Required,
            test_tags: vec!["conformance".to_string()],
        });
        c.required_semantics.push(Semantic {
            id: "req-missing-key".to_string(),
            description: "get(key) returns None for missing keys".to_string(),
            test_tags: vec!["basic".to_string()],
        });
        c.forbidden_semantics.push(Semantic {
            id: "forbid-panic-on-missing".to_string(),
            description: "Must not panic on missing key lookup".to_string(),
            test_tags: vec!["safety".to_string()],
        });
        c
    }

    #[test]
    fn test_trait_based_generation() {
        let code = generate_conformance_code(&sample_contract(), ConformanceStyle::TraitBased);
        assert!(code.contains("trait"));
        assert!(code.contains("inv_get_after_set"));
        assert!(code.contains("req_missing_key"));
        assert!(code.contains("forbid_panic_on_missing"));
    }

    #[test]
    fn test_factory_based_generation() {
        let code = generate_conformance_code(&sample_contract(), ConformanceStyle::FactoryBased);
        assert!(code.contains("fn create_instance"));
        assert!(code.contains("inv_get_after_set"));
    }
}
