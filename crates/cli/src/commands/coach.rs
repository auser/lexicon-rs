use lexicon_conversation::driver::TerminalDriver;
use lexicon_core::coach::{CoachTarget, run_coach};
use lexicon_repo::layout::RepoLayout;

use crate::app::CoachAction;
use crate::output;

pub fn run(action: Option<CoachAction>) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;
    let driver = TerminalDriver;

    let target = match action {
        Some(CoachAction::Contract { description }) => CoachTarget::Contract { description },
        Some(CoachAction::Conformance { contract_id }) => {
            CoachTarget::Conformance { contract_id }
        }
        Some(CoachAction::Prompt {
            contract_id,
            targets,
        }) => CoachTarget::Prompt {
            contract_id,
            targets,
        },
        Some(CoachAction::Improve) => CoachTarget::Improve,
        None => CoachTarget::OpenEnded,
    };

    output::heading("Coach Mode");
    let result = run_coach(&layout, target, &driver)?;

    output::divider();
    if result.accepted.is_empty() {
        output::info("No artifacts accepted.");
    } else {
        output::success(&format!(
            "{} artifact(s) accepted.",
            result.accepted.len()
        ));
        for a in &result.accepted {
            output::info(&format!("  {} ({})", a.path, a.format));
        }
    }
    Ok(())
}
