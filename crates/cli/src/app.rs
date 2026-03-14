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

    /// AI-powered architecture design session
    Chat,

    /// Interactive AI-assisted artifact coaching
    Coach {
        #[command(subcommand)]
        action: Option<CoachAction>,
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

    /// Workspace governance commands
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },

    /// Ecosystem governance commands
    Ecosystem {
        #[command(subcommand)]
        action: EcosystemAction,
    },

    /// Manage implementation prompts
    Prompt {
        #[command(subcommand)]
        action: PromptAction,
    },

    /// Launch the terminal UI
    Tui,
}

#[derive(Subcommand)]
pub enum WorkspaceAction {
    /// Initialize workspace governance
    Init,
    /// Verify workspace architecture
    Verify,
    /// Check workspace health
    Doctor,
}

#[derive(Subcommand)]
pub enum EcosystemAction {
    /// Initialize ecosystem governance
    Init,
    /// Verify ecosystem governance
    Verify,
    /// Check ecosystem health
    Doctor,
}

#[derive(Subcommand)]
pub enum ContractAction {
    /// Create a new contract (interactive unless --title is provided)
    New {
        /// Contract title (e.g. "Key-Value Store"). Enables non-interactive mode.
        #[arg(long)]
        title: Option<String>,
        /// Contract description. AI-generated from title if omitted.
        #[arg(long)]
        description: Option<String>,
        /// Comma-separated scope descriptions. AI-generated from title if omitted.
        #[arg(long)]
        scope: Option<String>,
        /// Comma-separated invariants
        #[arg(long)]
        invariants: Option<String>,
        /// Comma-separated required semantics
        #[arg(long)]
        required: Option<String>,
        /// Comma-separated forbidden behaviors
        #[arg(long)]
        forbidden: Option<String>,
    },
    /// List all contracts
    List,
    /// Lint contracts for issues
    Lint,
    /// AI-generate a contract from a description
    Generate {
        /// Natural language description (e.g. "async key-value store with TTL")
        intent: String,
    },
    /// Infer a contract from the public API surface
    Infer {
        /// Path to source directory (defaults to src/)
        #[arg(long)]
        path: Option<String>,
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
    /// Generate conformance tests from an existing contract
    FromContract {
        /// Contract ID (e.g. "key-value-store")
        contract_id: String,
    },
    /// Generate property-based tests from a contract's invariants
    Property {
        /// Contract ID
        contract_id: String,
    },
    /// Generate a fuzz test harness from a contract
    Fuzz {
        /// Contract ID
        contract_id: String,
    },
    /// Generate edge case tests from a contract
    EdgeCases {
        /// Contract ID
        contract_id: String,
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
    /// AI-generate tests to fill coverage gaps
    Improve {
        /// Only improve coverage for this contract
        #[arg(long)]
        contract: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Sync CLAUDE.md with current repo state
    Claude,
}

#[derive(Subcommand)]
pub enum PromptAction {
    /// Generate an implementation prompt from a contract
    Generate {
        /// Contract ID (e.g., "blob-store")
        contract_id: String,
        /// Optional target focus (e.g., "memory", "sqlite")
        #[arg(long)]
        target: Option<String>,
        /// Use AI to enhance the prompt
        #[arg(long)]
        ai: bool,
    },
    /// List existing prompts
    List,
    /// Show synchronization status
    Status,
    /// Regenerate stale prompts (or a specific one)
    Regenerate {
        /// Specific prompt (e.g., "001-memory-blob-store"). All stale if omitted.
        prompt: Option<String>,
        /// Regenerate all, not just stale
        #[arg(long)]
        all: bool,
        /// Use AI to enhance the prompt
        #[arg(long)]
        ai: bool,
    },
    /// Explain dependency chain for a prompt
    Explain {
        /// Prompt name (e.g., "001-memory-blob-store")
        prompt: String,
    },
}

#[derive(Subcommand)]
pub enum CoachAction {
    /// Coach a new contract from a description
    Contract {
        /// Natural language description (e.g. "async blob store with metadata")
        description: String,
    },
    /// Coach conformance tests for a contract
    Conformance {
        /// Contract ID (e.g. "blob-store")
        contract_id: String,
    },
    /// Coach implementation prompts for a contract
    Prompt {
        /// Contract ID
        contract_id: String,
        /// Comma-separated targets (e.g. "memory,file")
        #[arg(long)]
        targets: Option<String>,
    },
    /// Coach general improvements
    Improve,
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
    /// Store an API key directly (instead of browser OAuth)
    SetKey {
        /// Provider name (claude, openai)
        provider: lexicon_spec::auth::Provider,
        /// The API key (e.g. sk-ant-...)
        key: String,
    },
    /// Remove stored credentials for a provider
    Logout {
        /// Provider name (claude, openai). Interactive if omitted.
        provider: Option<lexicon_spec::auth::Provider>,
    },
}
