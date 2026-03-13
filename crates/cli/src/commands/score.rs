use lexicon_core::score::{gate_init, score_explain, score_init};
use lexicon_repo::layout::RepoLayout;

use crate::app::ScoreAction;
use crate::output;

pub fn run(action: ScoreAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        ScoreAction::Init => {
            score_init(&layout)?;
            output::success("Scoring model initialized");
        }
        ScoreAction::Explain => {
            let explanation = score_explain(&layout)?;
            println!("{explanation}");
        }
    }
    Ok(())
}

pub fn run_gate_init() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;
    gate_init(&layout)?;
    output::success("Gates initialized");
    Ok(())
}
