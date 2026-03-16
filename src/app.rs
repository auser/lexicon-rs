use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lexicon",
    about = "Contract-driven verification for Rust libraries and workspaces",
    version
)]
pub struct Cli {
    /// Override the AI model (e.g. claude-sonnet-4-20250514)
    #[arg(long, global = true)]
    pub model: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize lexicon in the current repository
    Init,

    /// AI-powered architecture design session (default when no command given)
    Chat,

    /// Run verification (gates + scoring + coverage + API drift)
    Verify {
        /// Include repo health checks (manifest, gates, scoring, CLAUDE.md)
        #[arg(long)]
        health: bool,
    },

    /// Manage AI provider authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Launch the terminal UI
    Tui,
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Login to an AI provider via browser OAuth
    Login {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_rs::spec::auth::Provider>,
        /// Custom OAuth callback port
        #[arg(long)]
        port: Option<u16>,
    },
    /// Refresh an expired OAuth token
    Refresh {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_rs::spec::auth::Provider>,
    },
    /// Show authentication status for all providers
    Status,
    /// Store an API key directly (instead of browser OAuth)
    SetKey {
        /// Provider name (claude, openai)
        provider: lexicon_rs::spec::auth::Provider,
        /// The API key (e.g. sk-ant-...)
        key: String,
    },
    /// Remove stored credentials for a provider
    Logout {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_rs::spec::auth::Provider>,
    },
}
