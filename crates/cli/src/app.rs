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

    /// AI-powered architecture design session
    Chat,

    /// Run verification (gates + scoring)
    Verify,

    /// Manage AI provider authentication
    Auth {
        #[command(subcommand)]
        action: AuthAction,
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
