use lexicon_core::sync_claude::sync_claude;
use lexicon_repo::layout::RepoLayout;

use crate::app::SyncAction;
use crate::output;

pub fn run(action: SyncAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        SyncAction::Claude => {
            sync_claude(&layout)?;
            output::success("CLAUDE.md synced");
        }
    }
    Ok(())
}
