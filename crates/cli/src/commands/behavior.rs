use lexicon_ai::prompt::ArtifactKind;
use lexicon_repo::layout::RepoLayout;

use crate::app::BehaviorAction;
use crate::output;

pub fn run(action: BehaviorAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        BehaviorAction::Add { contract_id: _ } => {
            output::warning("Behavior scenario creation not yet implemented");
            Ok(())
        }
        BehaviorAction::Sync => {
            output::warning("Behavior sync not yet implemented");
            Ok(())
        }
        BehaviorAction::Generate { intent } => {
            crate::commands::generate::run_generate(
                &layout,
                ArtifactKind::Behavior,
                &intent,
            )
        }
    }
}
