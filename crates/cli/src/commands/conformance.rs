use lexicon_ai::prompt::ArtifactKind;
use lexicon_repo::layout::RepoLayout;

use crate::app::ConformanceAction;
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
    }
}
