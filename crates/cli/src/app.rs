use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lexicon",
    about = "Contract-driven verification for Rust libraries and workspaces",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize lexicon in the current repository
    Init,

    /// Manage contracts
    Contract {
        #[command(subcommand)]
        action: ContractAction,
    },

    /// Manage conformance suites
    Conformance {
        #[command(subcommand)]
        action: ConformanceAction,
    },

    /// Manage behavior scenarios
    Behavior {
        #[command(subcommand)]
        action: BehaviorAction,
    },

    /// Manage the scoring model
    Score {
        #[command(subcommand)]
        action: ScoreAction,
    },

    /// Manage gates
    Gate {
        #[command(subcommand)]
        action: GateAction,
    },

    /// Run verification (gates + scoring)
    Verify,

    /// AI-guided improvement loop
    Improve {
        /// Goal to optimize for
        #[arg(long)]
        goal: Option<String>,
    },

    /// Check repo health and detect drift
    Doctor,

    /// Sync generated files
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },

    /// Launch the terminal UI
    Tui,
}

#[derive(Subcommand)]
pub enum ContractAction {
    /// Create a new contract interactively
    New,
    /// List all contracts
    List,
    /// Lint contracts for issues
    Lint,
}

#[derive(Subcommand)]
pub enum ConformanceAction {
    /// Add a conformance suite to a contract
    Add {
        /// Contract ID
        contract_id: String,
    },
    /// Sync conformance harness code
    Sync,
}

#[derive(Subcommand)]
pub enum BehaviorAction {
    /// Add a behavior scenario
    Add {
        /// Contract ID
        contract_id: String,
    },
    /// Sync behavior scenarios
    Sync,
}

#[derive(Subcommand)]
pub enum ScoreAction {
    /// Initialize the default scoring model
    Init,
    /// Explain the current scoring model
    Explain,
}

#[derive(Subcommand)]
pub enum GateAction {
    /// Initialize the default gates
    Init,
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Sync CLAUDE.md with current repo state
    Claude,
}
