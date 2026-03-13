use lexicon_core::ecosystem_mgmt;
use lexicon_repo::layout::RepoLayout;

use crate::app::EcosystemAction;
use crate::output;

pub fn run(action: EcosystemAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        EcosystemAction::Init => {
            output::heading("Initializing ecosystem governance");
            ecosystem_mgmt::ecosystem_init(&layout)
                .map_err(|e| miette::miette!("{e}"))?;
            output::success("Ecosystem governance initialized");
            output::info("Created .lexicon/ecosystem.toml");
            output::info("Created .lexicon/ecosystem/ directory");
            output::info("Add repos with `lexicon ecosystem verify` after editing ecosystem.toml");
        }
        EcosystemAction::Verify => {
            output::heading("Verifying ecosystem governance");
            let result = ecosystem_mgmt::ecosystem_verify(&layout)
                .map_err(|e| miette::miette!("{e}"))?;

            if result.passed {
                output::success("Ecosystem verification passed");
            } else {
                for issue in &result.issues {
                    output::warning(issue);
                }
                output::error("Ecosystem verification found issues");
            }
        }
        EcosystemAction::Doctor => {
            output::heading("Ecosystem health check");
            let issues = ecosystem_mgmt::ecosystem_doctor(&layout)
                .map_err(|e| miette::miette!("{e}"))?;

            if issues.is_empty() {
                output::success("Ecosystem is healthy");
            } else {
                for issue in &issues {
                    output::warning(issue);
                }
            }
        }
    }

    Ok(())
}
