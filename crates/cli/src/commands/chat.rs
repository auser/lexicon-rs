use lexicon_conversation::driver::TerminalDriver;
use lexicon_repo::layout::RepoLayout;

use crate::output;

pub fn run() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;
    let driver = TerminalDriver;

    output::heading("Lexicon Design Session");

    lexicon_core::chat::run_chat(&layout, &driver)
        .map_err(|e| miette::miette!("{e}"))?;

    output::divider();
    output::info("Design session ended.");
    Ok(())
}
