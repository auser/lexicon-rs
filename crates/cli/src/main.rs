mod app;
mod commands;
mod output;

use clap::Parser;

use app::{Cli, Command};

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let model = cli.model.as_deref();

    match cli.command {
        None | Some(Command::Chat) => commands::chat::run(model),
        Some(Command::Init) => commands::init::run(),
        Some(Command::Verify { health }) => commands::verify::run(health),
        Some(Command::Auth { action }) => commands::auth::run(action),
        Some(Command::Tui) => {
            let layout = lexicon_repo::layout::RepoLayout::discover()?;
            lexicon_tui::run_tui(layout)
                .map_err(|e| miette::miette!("TUI error: {e}"))?;
            Ok(())
        }
    }
}
