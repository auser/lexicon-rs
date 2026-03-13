use lexicon_core::contract::contract_list;
use lexicon_core::coverage;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::contract::Contract;

use crate::app::CoverageAction;
use crate::output;

pub fn run(action: CoverageAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        CoverageAction::Report { json } => {
            output::heading("Contract Coverage Report");

            // Load contracts
            let contract_ids = contract_list(&layout)?;
            if contract_ids.is_empty() {
                output::warning("No contracts found");
                return Ok(());
            }

            let contracts = load_contracts(&layout, &contract_ids);
            let report = coverage::coverage_report(&layout, &contracts)?;

            if json {
                let json_str = coverage::coverage_report_json(&report)?;
                println!("{json_str}");
            } else {
                let text = coverage::coverage_report_text(&report);
                println!("{text}");
            }
        }
    }
    Ok(())
}

fn load_contracts(layout: &RepoLayout, ids: &[String]) -> Vec<Contract> {
    let mut contracts = Vec::new();
    for id in ids {
        let path = layout.contracts_dir().join(format!("{id}.toml"));
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(contract) = toml::from_str::<Contract>(&content) {
                contracts.push(contract);
            }
        }
    }
    contracts
}
