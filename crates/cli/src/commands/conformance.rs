use lexicon_ai::prompt::ArtifactKind;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::contract::Contract;

use crate::app::ConformanceAction;
use crate::commands::review::{review_artifact, show_warnings};
use crate::output;

pub fn run(action: ConformanceAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        ConformanceAction::Add { contract_id: _ } => {
            output::warning("Conformance suite creation not yet implemented");
            Ok(())
        }
        ConformanceAction::Sync => {
            output::warning("Conformance sync not yet implemented");
            Ok(())
        }
        ConformanceAction::Generate { intent } => {
            crate::commands::generate::run_generate(
                &layout,
                ArtifactKind::Conformance,
                &intent,
            )
        }
        ConformanceAction::FromContract { contract_id } => {
            let contract = load_contract(&layout, &contract_id)?;
            output::heading("Generate Conformance Tests from Contract");
            output::info(&format!("Contract: {} — {}", contract.id, contract.title));
            output::divider();

            let result = lexicon_core::generate::generate_tests_from_contract(&layout, &contract)?;
            show_warnings(&result.warnings);
            review_artifact(&layout, &result.artifact)?;
            Ok(())
        }
        ConformanceAction::Property { contract_id } => {
            let contract = load_contract(&layout, &contract_id)?;
            output::heading("Generate Property Tests from Contract");
            output::info(&format!("Contract: {} — {}", contract.id, contract.title));
            output::info(&format!("Invariants: {}", contract.invariants.len()));
            output::divider();

            let result = lexicon_core::generate::generate_contract_property_tests(&layout, &contract)?;
            show_warnings(&result.warnings);
            review_artifact(&layout, &result.artifact)?;
            Ok(())
        }
        ConformanceAction::Fuzz { contract_id } => {
            let contract = load_contract(&layout, &contract_id)?;
            output::heading("Generate Fuzz Target from Contract");
            output::info(&format!("Contract: {} — {}", contract.id, contract.title));
            output::divider();

            let result = lexicon_core::generate::generate_contract_fuzz_target(&layout, &contract)?;
            show_warnings(&result.warnings);
            review_artifact(&layout, &result.artifact)?;
            Ok(())
        }
        ConformanceAction::EdgeCases { contract_id } => {
            let contract = load_contract(&layout, &contract_id)?;
            output::heading("Generate Edge Case Tests from Contract");
            output::info(&format!("Contract: {} — {}", contract.id, contract.title));
            output::info(&format!("Edge cases: {}", contract.edge_cases.len()));
            output::divider();

            let result = lexicon_core::generate::generate_contract_edge_case_tests(&layout, &contract)?;
            show_warnings(&result.warnings);
            review_artifact(&layout, &result.artifact)?;
            Ok(())
        }
    }
}

fn load_contract(layout: &RepoLayout, contract_id: &str) -> miette::Result<Contract> {
    let path = layout.contracts_dir().join(format!("{contract_id}.toml"));
    let content = std::fs::read_to_string(&path)
        .map_err(|_| miette::miette!("Contract '{contract_id}' not found at {}", path.display()))?;
    let contract: Contract = toml::from_str(&content)
        .map_err(|e| miette::miette!("Failed to parse contract '{contract_id}': {e}"))?;
    Ok(contract)
}
