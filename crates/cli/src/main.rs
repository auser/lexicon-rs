mod app;
mod commands;
mod output;

use clap::Parser;

use app::{Cli, Command};

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => commands::init::run(),
        Command::Contract { action } => commands::contract::run(action),
        Command::Conformance { action } => commands::conformance::run(action),
        Command::Behavior { action } => commands::behavior::run(action),
        Command::Score { action } => commands::score::run(action),
        Command::Gate { action } => commands::gate::run(action),
        Command::Api { action } => commands::api::run(action),
        Command::Coverage { action } => commands::coverage::run(action),
        Command::Verify => commands::verify::run(),
        Command::Auth { action } => commands::auth::run(action),
        Command::Improve { goal } => commands::generate::run_improve(goal.as_deref()),
        Command::Generate { intent } => commands::generate::run(&intent),
        Command::Workspace { action } => commands::workspace_cmd::run(action),
        Command::Ecosystem { action } => commands::ecosystem_cmd::run(action),
        Command::Doctor => commands::doctor::run(),
        Command::Sync { action } => commands::sync::run(action),
        Command::Tui => {
            let layout = lexicon_repo::layout::RepoLayout::discover()?;
            lexicon_tui::run_tui(layout)
                .map_err(|e| miette::miette!("TUI error: {e}"))?;
            Ok(())
        }
    }
}
