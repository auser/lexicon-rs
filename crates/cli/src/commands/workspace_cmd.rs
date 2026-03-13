use lexicon_core::workspace;
use lexicon_repo::layout::RepoLayout;

use crate::app::WorkspaceAction;
use crate::output;

pub fn run(action: WorkspaceAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        WorkspaceAction::Init => {
            output::heading("Initializing workspace governance");
            workspace::workspace_init(&layout)
                .map_err(|e| miette::miette!("{e}"))?;
            output::success("Workspace governance initialized");
            output::info("Created .lexicon/workspace.toml with detected crate roles");
            output::info("Created .lexicon/architecture/rules.toml with default dependency rules");
            output::info("Run `lexicon workspace verify` to check workspace architecture");
        }
        WorkspaceAction::Verify => {
            output::heading("Verifying workspace architecture");
            let result = workspace::workspace_verify(&layout)
                .map_err(|e| miette::miette!("{e}"))?;

            if result.passed {
                output::success("Workspace verification passed");
            } else {
                for issue in &result.issues {
                    output::warning(issue);
                }
                output::error("Workspace verification found issues");
            }
        }
        WorkspaceAction::Doctor => {
            output::heading("Workspace health check");
            let issues = workspace::workspace_doctor(&layout)
                .map_err(|e| miette::miette!("{e}"))?;

            if issues.is_empty() {
                output::success("Workspace is healthy");
            } else {
                for issue in &issues {
                    output::warning(issue);
                }
            }
        }
    }

    Ok(())
}
