use lexicon_repo::layout::RepoLayout;
use lexicon_spec::common::ConformanceStyle;
use lexicon_spec::contract::Contract;

use crate::error::ScaffoldResult;

/// Write generated conformance test code to the conformance tests directory.
pub fn write_conformance_harness(
    layout: &RepoLayout,
    contract: &Contract,
    style: ConformanceStyle,
) -> ScaffoldResult<()> {
    let code = lexicon_conformance::generator::generate_conformance_code(contract, style);
    let filename = format!("{}.rs", contract.id.replace('-', "_"));
    let path = layout.conformance_tests_dir().join(filename);
    lexicon_fs::safe_write::safe_write(&path, &code, false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::common::Severity;
    use lexicon_spec::contract::Invariant;
    use tempfile::TempDir;

    #[test]
    fn test_write_conformance_harness() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        std::fs::create_dir_all(layout.conformance_tests_dir()).unwrap();

        let mut contract = Contract::new_draft(
            "kv-store".to_string(),
            "KV Store".to_string(),
            "KV".to_string(),
        );
        contract.invariants.push(Invariant {
            id: "inv-001".to_string(),
            description: "get after set".to_string(),
            severity: Severity::Required,
            test_tags: vec!["conformance".to_string()],
        });

        write_conformance_harness(&layout, &contract, ConformanceStyle::TraitBased).unwrap();

        let path = layout.conformance_tests_dir().join("kv_store.rs");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("inv_001"));
    }
}
