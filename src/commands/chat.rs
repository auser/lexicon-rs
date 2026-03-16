use lexicon_rs::conversation::driver::TerminalDriver;
use lexicon_rs::repo::layout::RepoLayout;

use crate::output;

pub fn run(model: Option<&str>) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;
    let driver = TerminalDriver;

    output::heading("Lexicon Design Session");

    lexicon_rs::core::chat::run_chat(&layout, &driver, model)
        .map_err(|e| miette::miette!("{e}"))?;

    output::divider();
    output::info("Design session ended.");
    Ok(())
}
