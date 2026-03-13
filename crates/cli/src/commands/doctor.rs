use lexicon_repo::layout::RepoLayout;

use crate::output;

pub fn run() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("Doctor — checking repo health");

    // Check manifest
    if layout.manifest_path().exists() {
        output::success("Manifest found");
    } else {
        output::error("No manifest — run `lexicon init` first");
        return Ok(());
    }

    // Check contracts dir
    let contracts = lexicon_core::contract::contract_list(&layout)?;
    output::info(&format!("{} contract(s)", contracts.len()));

    // Check scoring model
    if layout.scoring_model_path().exists() {
        output::success("Scoring model configured");
    } else {
        output::warning("No scoring model — run `lexicon score init`");
    }

    // Check gates
    if layout.gates_path().exists() {
        output::success("Gates configured");
    } else {
        output::warning("No gates — run `lexicon gate init`");
    }

    // Check CLAUDE.md
    if layout.claude_md_path().exists() {
        output::success("CLAUDE.md present");
    } else {
        output::warning("No CLAUDE.md — run `lexicon sync claude`");
    }

    Ok(())
}
