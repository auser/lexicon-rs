use lexicon_conversation::driver::TerminalDriver;
use lexicon_core::contract::{contract_list, contract_new};
use lexicon_repo::layout::RepoLayout;

use crate::app::ContractAction;
use crate::output;

pub fn run(action: ContractAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        ContractAction::New => {
            output::heading("New Contract");
            let driver = TerminalDriver;
            let result = contract_new(&layout, &driver)?;
            match result {
                Some(c) => output::success(&format!("Contract '{}' created", c.id)),
                None => output::warning("Contract creation cancelled"),
            }
        }
        ContractAction::List => {
            let ids = contract_list(&layout)?;
            if ids.is_empty() {
                output::info("No contracts found. Run `lexicon contract new` to create one.");
            } else {
                output::heading("Contracts");
                for id in &ids {
                    output::info(id);
                }
            }
        }
        ContractAction::Lint => {
            output::warning("Contract linting not yet implemented");
        }
        ContractAction::Generate { intent } => {
            return crate::commands::generate::run_generate(
                &layout,
                lexicon_ai::prompt::ArtifactKind::Contract,
                &intent,
            );
        }
        ContractAction::Infer { path } => {
            output::heading("Infer Contract from API");
            output::info("Scanning public API surface...");
            output::divider();

            let source_dir = path.as_ref().map(std::path::Path::new);
            let result = lexicon_core::generate::infer_contract_from_api(&layout, source_dir)?;
            crate::commands::review::show_warnings(&result.warnings);
            crate::commands::review::review_artifact(&layout, &result.artifact)?;
        }
    }
    Ok(())
}
