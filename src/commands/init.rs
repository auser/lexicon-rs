use lexicon_rs::conversation::driver::TerminalDriver;
use lexicon_rs::core::init::init_repo;
use lexicon_rs::repo::layout::RepoLayout;

use crate::output;

pub fn run() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("Initializing lexicon");
    let driver = TerminalDriver;
    init_repo(&layout, &driver)?;
    output::success("Lexicon initialized successfully (with default gates and scoring model)");
    Ok(())
}
