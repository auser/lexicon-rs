use lexicon_conversation::driver::TerminalDriver;
use lexicon_core::init::init_repo;
use lexicon_repo::layout::RepoLayout;

use crate::output;

pub fn run() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("Initializing lexicon");
    let driver = TerminalDriver;
    init_repo(&layout, &driver)?;
    output::success("Lexicon initialized successfully");
    Ok(())
}
