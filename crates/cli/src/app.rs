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

    /// Manage AI provider authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// AI-guided improvement loop
    Improve {
        /// Goal to optimize for
        #[arg(long)]
        goal: Option<String>,
    },

    /// Generate artifacts from a natural language description
    Generate {
        /// What to generate (e.g. "rate limiter with burst capacity")
        intent: String,
    },

    /// Manage public API extraction and drift
    Api {
        #[command(subcommand)]
        action: ApiAction,
    },

    /// Analyze contract test coverage
    Coverage {
        #[command(subcommand)]
        action: CoverageAction,
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
    /// AI-generate a contract from a description
    Generate {
        /// Natural language description (e.g. "async key-value store with TTL")
        intent: String,
    },
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
    /// AI-generate conformance tests from a description
    Generate {
        /// Natural language description (e.g. "cache trait with async get/set/delete")
        intent: String,
    },
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
    /// AI-generate behavior scenarios from a description
    Generate {
        /// Natural language description (e.g. "user session expiration after inactivity")
        intent: String,
    },
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
pub enum ApiAction {
    /// Scan and extract the public API
    Scan,
    /// Compare current API against baseline
    Diff,
    /// Show API diff report
    Report {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Save current API as baseline
    Baseline,
}

#[derive(Subcommand)]
pub enum CoverageAction {
    /// Show contract coverage report
    Report {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Sync CLAUDE.md with current repo state
    Claude,
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Login to an AI provider via browser OAuth
    Login {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_spec::auth::Provider>,
        /// Custom OAuth callback port
        #[arg(long)]
        port: Option<u16>,
    },
    /// Refresh an expired OAuth token
    Refresh {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_spec::auth::Provider>,
    },
    /// Show authentication status for all providers
    Status,
    /// Remove stored credentials for a provider
    Logout {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_spec::auth::Provider>,
    },
}
