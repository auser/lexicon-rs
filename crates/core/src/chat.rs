//! AI-guided architecture design session.
//!
//! Provides a conversational REPL where an AI agent drives the user through
//! designing contracts, gates, conformance tests, and other artifacts, then
//! compiles everything into a constraint-aware implementation prompt.

use lexicon_ai::boundary::AiProvider;
use lexicon_ai::generate::load_context;
use lexicon_ai::prompt::{build_chat_user_message, CHAT_SYSTEM};
use lexicon_conversation::driver::ConversationDriver;
use lexicon_conversation::session::{list_sessions, save_session};
use lexicon_conversation::workflow::Question;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::common::{StepType, WorkflowKind};
use lexicon_spec::contract::Contract;
use lexicon_spec::session::ConversationSession;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::error::{CoreError, CoreResult};
use crate::generate::build_ai_provider;

/// Maximum conversation turns in history sent to the AI (to manage context window).
const MAX_HISTORY_TURNS: usize = 15;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Category of architecture artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactCategory {
    Contract,
    Conformance,
    Behavior,
    Gate,
    Score,
    Prompt,
}

impl std::fmt::Display for ArtifactCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contract => write!(f, "Contract"),
            Self::Conformance => write!(f, "Conformance"),
            Self::Behavior => write!(f, "Behavior"),
            Self::Gate => write!(f, "Gate"),
            Self::Score => write!(f, "Score"),
            Self::Prompt => write!(f, "Prompt"),
        }
    }
}

/// An artifact created or modified during the chat session.
#[derive(Debug, Clone)]
pub struct SessionArtifact {
    pub kind: ArtifactCategory,
    pub id: String,
    pub path: String,
    pub summary: String,
}

/// A single message in the conversation history.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Role of a chat message sender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Assistant => write!(f, "Assistant"),
        }
    }
}

/// Accumulated state during a chat design session.
pub struct ChatContext {
    pub artifacts: Vec<SessionArtifact>,
    pub history: Vec<ChatMessage>,
}

impl ChatContext {
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
            history: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Action parsing
// ---------------------------------------------------------------------------

/// An action directive parsed from an AI response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAction {
    CreateContract { toml_content: String },
    UpdateContract { id: String, toml_content: String },
    CreateGate { toml_content: String },
    CreateConformance { contract_id: String },
    CreateBehavior { contract_id: String },
    CreatePropertyTests { contract_id: String },
    CreateFuzzTarget { contract_id: String },
    CreateEdgeCases { contract_id: String },
    InferContract,
    CoverageReport,
    ApiScan,
    ApiBaseline,
    SyncClaude,
    Doctor,
    PromptList,
    PromptStatus,
    RegeneratePrompts,
    GeneratePrompt,
    RunVerify,
}

impl ChatAction {
    /// Human-readable label for progress display.
    fn label(&self) -> &'static str {
        match self {
            Self::CreateContract { .. } => "Creating contract...",
            Self::UpdateContract { .. } => "Updating contract...",
            Self::CreateGate { .. } => "Creating gate...",
            Self::CreateConformance { .. } => "Generating conformance tests...",
            Self::CreateBehavior { .. } => "Generating behavior scenarios...",
            Self::CreatePropertyTests { .. } => "Generating property tests...",
            Self::CreateFuzzTarget { .. } => "Generating fuzz target...",
            Self::CreateEdgeCases { .. } => "Generating edge case tests...",
            Self::InferContract => "Inferring contract from API...",
            Self::CoverageReport => "Analyzing coverage...",
            Self::ApiScan => "Scanning public API...",
            Self::ApiBaseline => "Saving API baseline...",
            Self::SyncClaude => "Syncing CLAUDE.md...",
            Self::Doctor => "Checking repo health...",
            Self::PromptList => "Listing prompts...",
            Self::PromptStatus => "Checking prompt status...",
            Self::RegeneratePrompts => "Regenerating prompts...",
            Self::GeneratePrompt => "Compiling implementation prompt...",
            Self::RunVerify => "Running verification...",
        }
    }
}

/// Parsed AI response with conversational text and any action directives.
pub struct ParsedResponse {
    pub display_text: String,
    pub actions: Vec<ChatAction>,
}

/// Parse an AI response for `:::ACTION ... :::` directive blocks.
pub fn parse_ai_response(response: &str) -> ParsedResponse {
    let mut display_text = String::new();
    let mut actions = Vec::new();

    let mut lines = response.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(directive) = line.strip_prefix(":::ACTION ") {
            let directive = directive.trim();
            let mut block_content = String::new();

            // Collect lines until closing :::
            for inner in lines.by_ref() {
                if inner.trim() == ":::" {
                    break;
                }
                if !block_content.is_empty() {
                    block_content.push('\n');
                }
                block_content.push_str(inner);
            }

            if let Some(action) = parse_directive(directive, &block_content) {
                actions.push(action);
            }
        } else {
            if !display_text.is_empty() {
                display_text.push('\n');
            }
            display_text.push_str(line);
        }
    }

    ParsedResponse {
        display_text: display_text.trim().to_string(),
        actions,
    }
}

/// Parse a single directive line + block content into a ChatAction.
fn parse_directive(directive: &str, content: &str) -> Option<ChatAction> {
    let parts: Vec<&str> = directive.splitn(2, ' ').collect();
    let command = parts[0];
    let arg = parts.get(1).map(|s| s.trim().to_string());

    match command {
        "CREATE_CONTRACT" => Some(ChatAction::CreateContract {
            toml_content: content.to_string(),
        }),
        "UPDATE_CONTRACT" => Some(ChatAction::UpdateContract {
            id: arg.unwrap_or_default(),
            toml_content: content.to_string(),
        }),
        "CREATE_GATE" => Some(ChatAction::CreateGate {
            toml_content: content.to_string(),
        }),
        "CREATE_CONFORMANCE" => Some(ChatAction::CreateConformance {
            contract_id: arg.unwrap_or_default(),
        }),
        "CREATE_BEHAVIOR" => Some(ChatAction::CreateBehavior {
            contract_id: arg.unwrap_or_default(),
        }),
        "CREATE_PROPERTY_TESTS" => Some(ChatAction::CreatePropertyTests {
            contract_id: arg.unwrap_or_default(),
        }),
        "CREATE_FUZZ_TARGET" => Some(ChatAction::CreateFuzzTarget {
            contract_id: arg.unwrap_or_default(),
        }),
        "CREATE_EDGE_CASES" => Some(ChatAction::CreateEdgeCases {
            contract_id: arg.unwrap_or_default(),
        }),
        "INFER_CONTRACT" => Some(ChatAction::InferContract),
        "COVERAGE_REPORT" => Some(ChatAction::CoverageReport),
        "API_SCAN" => Some(ChatAction::ApiScan),
        "API_BASELINE" => Some(ChatAction::ApiBaseline),
        "SYNC_CLAUDE" => Some(ChatAction::SyncClaude),
        "DOCTOR" => Some(ChatAction::Doctor),
        "PROMPT_LIST" => Some(ChatAction::PromptList),
        "PROMPT_STATUS" => Some(ChatAction::PromptStatus),
        "REGENERATE_PROMPTS" => Some(ChatAction::RegeneratePrompts),
        "GENERATE_PROMPT" => Some(ChatAction::GeneratePrompt),
        "RUN_VERIFY" => Some(ChatAction::RunVerify),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Action execution
// ---------------------------------------------------------------------------

/// Execute a chat action, returning a summary of what was done.
fn execute_action(
    layout: &RepoLayout,
    action: &ChatAction,
    ctx: &mut ChatContext,
    ai_provider: &dyn AiProvider,
) -> CoreResult<String> {
    match action {
        ChatAction::CreateContract { toml_content } => {
            execute_create_contract(layout, toml_content, ctx)
        }
        ChatAction::UpdateContract { id, toml_content } => {
            execute_update_contract(layout, id, toml_content, ctx)
        }
        ChatAction::CreateGate { toml_content } => {
            execute_create_gate(layout, toml_content, ctx)
        }
        ChatAction::CreateConformance { contract_id } => {
            execute_create_conformance(layout, contract_id, ctx, ai_provider)
        }
        ChatAction::CreateBehavior { contract_id } => {
            execute_create_behavior(layout, contract_id, ctx, ai_provider)
        }
        ChatAction::CreatePropertyTests { contract_id } => {
            execute_create_property_tests(layout, contract_id, ctx)
        }
        ChatAction::CreateFuzzTarget { contract_id } => {
            execute_create_fuzz_target(layout, contract_id, ctx)
        }
        ChatAction::CreateEdgeCases { contract_id } => {
            execute_create_edge_cases(layout, contract_id, ctx)
        }
        ChatAction::InferContract => execute_infer_contract(layout, ctx),
        ChatAction::CoverageReport => execute_coverage_report(layout),
        ChatAction::ApiScan => execute_api_scan(layout),
        ChatAction::ApiBaseline => execute_api_baseline(layout),
        ChatAction::SyncClaude => execute_sync_claude(layout),
        ChatAction::Doctor => execute_doctor(layout),
        ChatAction::PromptList => execute_prompt_list(layout),
        ChatAction::PromptStatus => execute_prompt_status(layout),
        ChatAction::RegeneratePrompts => execute_regenerate_prompts(layout),
        ChatAction::GeneratePrompt => execute_generate_prompt(layout, ctx),
        ChatAction::RunVerify => execute_verify(layout),
    }
}

fn execute_create_contract(
    layout: &RepoLayout,
    toml_content: &str,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    // Parse to extract the contract ID
    let contract: lexicon_spec::contract::Contract =
        toml::from_str(toml_content).map_err(|e| {
            CoreError::Other(format!("Failed to parse contract TOML from AI: {e}"))
        })?;

    let id = contract.id.clone();
    let title = contract.title.clone();
    let contracts_dir = layout.contracts_dir();
    std::fs::create_dir_all(&contracts_dir)?;
    let path = contracts_dir.join(format!("{id}.toml"));
    std::fs::write(&path, toml_content)?;

    let rel_path = format!("specs/contracts/{id}.toml");
    let inv_count = contract.invariants.len();
    let req_count = contract.required_semantics.len();
    let forbid_count = contract.forbidden_semantics.len();
    let edge_count = contract.edge_cases.len();

    let summary = format!(
        "{title}: {inv_count} invariants, {req_count} required, {forbid_count} forbidden, {edge_count} edge cases"
    );

    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Contract,
        id: id.clone(),
        path: rel_path.clone(),
        summary: summary.clone(),
    });

    Ok(format!(
        "✓ Created contract \"{id}\" at {rel_path}\n  {summary}"
    ))
}

fn execute_update_contract(
    layout: &RepoLayout,
    id: &str,
    toml_content: &str,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    // Validate the TOML
    let contract: lexicon_spec::contract::Contract =
        toml::from_str(toml_content).map_err(|e| {
            CoreError::Other(format!("Failed to parse updated contract TOML: {e}"))
        })?;

    let path = layout.contracts_dir().join(format!("{id}.toml"));
    if !path.exists() {
        return Err(CoreError::Other(format!("Contract not found: {id}")));
    }
    std::fs::write(&path, toml_content)?;

    let rel_path = format!("specs/contracts/{id}.toml");
    let summary = format!(
        "{}: {} invariants, {} required, {} forbidden, {} edge cases",
        contract.title,
        contract.invariants.len(),
        contract.required_semantics.len(),
        contract.forbidden_semantics.len(),
        contract.edge_cases.len()
    );

    // Update or add artifact in session
    if let Some(existing) = ctx.artifacts.iter_mut().find(|a| a.id == id) {
        existing.summary = summary.clone();
    } else {
        ctx.artifacts.push(SessionArtifact {
            kind: ArtifactCategory::Contract,
            id: id.to_string(),
            path: rel_path.clone(),
            summary: summary.clone(),
        });
    }

    Ok(format!(
        "✓ Updated contract \"{id}\" at {rel_path}\n  {summary}"
    ))
}

fn execute_create_gate(
    layout: &RepoLayout,
    toml_content: &str,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    let gates_path = layout.gates_path();

    // If gates file exists, we need to merge. For simplicity, overwrite with the full content.
    // The AI should provide the complete gates file.
    if let Some(parent) = gates_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&gates_path, toml_content)?;

    // Count gates
    let model: Result<lexicon_spec::gates::GatesModel, _> = toml::from_str(toml_content);
    let count = model.map(|m| m.gates.len()).unwrap_or(0);

    let summary = format!("{count} gate(s) defined");
    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Gate,
        id: "gates".to_string(),
        path: "specs/gates.toml".to_string(),
        summary: summary.clone(),
    });

    Ok(format!("✓ Wrote gates to specs/gates.toml\n  {summary}"))
}

fn execute_create_conformance(
    layout: &RepoLayout,
    contract_id: &str,
    ctx: &mut ChatContext,
    _ai_provider: &dyn AiProvider,
) -> CoreResult<String> {
    use lexicon_ai::prompt::ArtifactKind;

    let intent = format!(
        "Generate conformance tests for the \"{contract_id}\" contract"
    );
    let result = crate::generate::generate_from_intent(layout, ArtifactKind::Conformance, &intent)?;
    crate::generate::accept_artifact(layout, &result.artifact)?;

    let path = result.artifact.path.clone();
    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Conformance,
        id: contract_id.to_string(),
        path: path.clone(),
        summary: format!("conformance tests for {contract_id}"),
    });

    Ok(format!(
        "✓ Generated conformance tests at {path}"
    ))
}

fn execute_create_behavior(
    layout: &RepoLayout,
    contract_id: &str,
    ctx: &mut ChatContext,
    _ai_provider: &dyn AiProvider,
) -> CoreResult<String> {
    use lexicon_ai::prompt::ArtifactKind;

    let intent = format!(
        "Generate behavior scenarios for the \"{contract_id}\" contract"
    );
    let result = crate::generate::generate_from_intent(layout, ArtifactKind::Behavior, &intent)?;
    crate::generate::accept_artifact(layout, &result.artifact)?;

    let path = result.artifact.path.clone();
    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Behavior,
        id: contract_id.to_string(),
        path: path.clone(),
        summary: format!("behavior scenarios for {contract_id}"),
    });

    Ok(format!(
        "✓ Generated behavior scenarios at {path}"
    ))
}

fn execute_generate_prompt(
    layout: &RepoLayout,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    // Find all contracts created during this session
    let contract_ids: Vec<String> = ctx
        .artifacts
        .iter()
        .filter(|a| a.kind == ArtifactCategory::Contract)
        .map(|a| a.id.clone())
        .collect();

    if contract_ids.is_empty() {
        return Ok("No contracts found in this session. Create a contract first.".to_string());
    }

    let mut results = Vec::new();
    for contract_id in &contract_ids {
        match crate::prompt_gen::generate_prompt(layout, contract_id, None, false) {
            Ok(result) => {
                crate::generate::accept_artifact(layout, &result.artifact)?;
                let path = result.artifact.path.clone();
                ctx.artifacts.push(SessionArtifact {
                    kind: ArtifactCategory::Prompt,
                    id: contract_id.clone(),
                    path: path.clone(),
                    summary: format!("implementation prompt for {contract_id}"),
                });
                results.push(format!("✓ Generated prompt at {path}"));
            }
            Err(e) => {
                results.push(format!("✗ Failed to generate prompt for {contract_id}: {e}"));
            }
        }
    }

    Ok(results.join("\n"))
}

fn execute_verify(layout: &RepoLayout) -> CoreResult<String> {
    match crate::verify::verify(layout) {
        Ok(result) => {
            let mut output = String::new();
            output.push_str("Verification Results:\n");
            for gate in &result.gate_results {
                let status = if gate.passed() { "PASS" } else { "FAIL" };
                output.push_str(&format!("  [{status}] {}\n", gate.gate_id));
            }
            if let Some(ref score) = result.score_report {
                output.push_str(&format!(
                    "  Score: {:.0}%\n",
                    score.total_score * 100.0,
                ));
            }
            Ok(output)
        }
        Err(e) => Ok(format!("Verification failed: {e}")),
    }
}

fn execute_create_property_tests(
    layout: &RepoLayout,
    contract_id: &str,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    let contract = load_contract_by_id(layout, contract_id)?;
    let result = crate::generate::generate_contract_property_tests(layout, &contract)?;
    crate::generate::accept_artifact(layout, &result.artifact)?;

    let path = result.artifact.path.clone();
    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Conformance,
        id: format!("{contract_id}-property"),
        path: path.clone(),
        summary: format!("property tests for {contract_id}"),
    });

    Ok(format!("Generated property tests at {path}"))
}

fn execute_create_fuzz_target(
    layout: &RepoLayout,
    contract_id: &str,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    let contract = load_contract_by_id(layout, contract_id)?;
    let result = crate::generate::generate_contract_fuzz_target(layout, &contract)?;
    crate::generate::accept_artifact(layout, &result.artifact)?;

    let path = result.artifact.path.clone();
    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Conformance,
        id: format!("{contract_id}-fuzz"),
        path: path.clone(),
        summary: format!("fuzz target for {contract_id}"),
    });

    Ok(format!("Generated fuzz target at {path}"))
}

fn execute_create_edge_cases(
    layout: &RepoLayout,
    contract_id: &str,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    let contract = load_contract_by_id(layout, contract_id)?;
    let result = crate::generate::generate_contract_edge_case_tests(layout, &contract)?;
    crate::generate::accept_artifact(layout, &result.artifact)?;

    let path = result.artifact.path.clone();
    ctx.artifacts.push(SessionArtifact {
        kind: ArtifactCategory::Conformance,
        id: format!("{contract_id}-edge"),
        path: path.clone(),
        summary: format!("edge case tests for {contract_id}"),
    });

    Ok(format!("Generated edge case tests at {path}"))
}

fn execute_infer_contract(
    layout: &RepoLayout,
    ctx: &mut ChatContext,
) -> CoreResult<String> {
    let result = crate::generate::infer_contract_from_api(layout, None)?;
    crate::generate::accept_artifact(layout, &result.artifact)?;

    let path = result.artifact.path.clone();
    // Try to extract ID from the generated artifact content
    let id = if let Ok(contract) = toml::from_str::<lexicon_spec::contract::Contract>(&result.artifact.content) {
        let cid = contract.id.clone();
        let summary = format!(
            "{}: {} invariants, {} required, {} forbidden",
            contract.title,
            contract.invariants.len(),
            contract.required_semantics.len(),
            contract.forbidden_semantics.len(),
        );
        ctx.artifacts.push(SessionArtifact {
            kind: ArtifactCategory::Contract,
            id: cid.clone(),
            path: path.clone(),
            summary,
        });
        cid
    } else {
        ctx.artifacts.push(SessionArtifact {
            kind: ArtifactCategory::Contract,
            id: "inferred".to_string(),
            path: path.clone(),
            summary: "inferred from public API".to_string(),
        });
        "inferred".to_string()
    };

    Ok(format!("Inferred contract \"{id}\" at {path}"))
}

fn execute_coverage_report(layout: &RepoLayout) -> CoreResult<String> {
    let contracts = load_all_contracts(layout)?;
    if contracts.is_empty() {
        return Ok("No contracts found. Create a contract first.".to_string());
    }

    let report = crate::coverage::coverage_report(layout, &contracts)?;
    let text = crate::coverage::coverage_report_text(&report);
    Ok(text)
}

fn execute_api_scan(layout: &RepoLayout) -> CoreResult<String> {
    let snapshot = crate::api::api_scan(layout)?;
    let mut output = format!("API scan: {} items extracted\n", snapshot.items.len());

    // Try to show drift if baseline exists
    match crate::api::api_diff(layout) {
        Ok(diff) => {
            output.push_str(&diff.summary());
        }
        Err(_) => {
            output.push_str("No baseline found — run `lexicon api baseline` to set one.");
        }
    }

    Ok(output)
}

/// Load a single contract by ID from the contracts directory.
fn load_contract_by_id(layout: &RepoLayout, contract_id: &str) -> CoreResult<Contract> {
    let path = layout.contracts_dir().join(format!("{contract_id}.toml"));
    if !path.exists() {
        return Err(CoreError::Other(format!(
            "Contract not found: {contract_id}"
        )));
    }
    let text = std::fs::read_to_string(&path)?;
    let contract: Contract = toml::from_str(&text)
        .map_err(|e| CoreError::Other(format!("Failed to parse contract {contract_id}: {e}")))?;
    Ok(contract)
}

/// Load all contracts from the contracts directory.
fn load_all_contracts(layout: &RepoLayout) -> CoreResult<Vec<Contract>> {
    let dir = layout.contracts_dir();
    let mut contracts = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir)?.flatten() {
            if entry.path().extension().is_some_and(|e| e == "toml") {
                let text = std::fs::read_to_string(entry.path())?;
                let contract: Contract = toml::from_str(&text)
                    .map_err(|e| CoreError::Other(format!("Failed to parse contract: {e}")))?;
                contracts.push(contract);
            }
        }
    }
    Ok(contracts)
}

fn execute_api_baseline(layout: &RepoLayout) -> CoreResult<String> {
    // Ensure a current scan exists first
    if !layout.api_dir().join("current.json").exists() {
        crate::api::api_scan(layout)?;
    }
    crate::api::api_baseline(layout)?;
    Ok("API baseline saved".to_string())
}

fn execute_sync_claude(layout: &RepoLayout) -> CoreResult<String> {
    crate::sync_claude::sync_claude(layout)?;
    Ok("CLAUDE.md synced with current repo state".to_string())
}

fn execute_doctor(layout: &RepoLayout) -> CoreResult<String> {
    let mut output = String::new();

    if layout.manifest_path().exists() {
        output.push_str("✓ Manifest found\n");
    } else {
        output.push_str("✗ No manifest — run `lexicon init`\n");
    }

    let contracts = crate::contract::contract_list(layout)?;
    output.push_str(&format!("  {} contract(s)\n", contracts.len()));

    if layout.scoring_model_path().exists() {
        output.push_str("✓ Scoring model configured\n");
    } else {
        output.push_str("⚠ No scoring model\n");
    }

    if layout.gates_path().exists() {
        output.push_str("✓ Gates configured\n");
    } else {
        output.push_str("⚠ No gates\n");
    }

    if layout.claude_md_path().exists() {
        output.push_str("✓ CLAUDE.md present\n");
    } else {
        output.push_str("⚠ No CLAUDE.md\n");
    }

    if layout.api_dir().join("baseline.json").exists() {
        output.push_str("✓ API baseline configured\n");
    } else {
        output.push_str("  No API baseline\n");
    }

    Ok(output)
}

fn execute_prompt_list(layout: &RepoLayout) -> CoreResult<String> {
    let prompts = crate::prompt_gen::list_prompts(layout)?;
    if prompts.is_empty() {
        return Ok("No prompts generated yet.".to_string());
    }
    let mut output = format!("{} prompt(s):\n", prompts.len());
    for p in &prompts {
        output.push_str(&format!("  - {p}\n"));
    }
    Ok(output)
}

fn execute_prompt_status(layout: &RepoLayout) -> CoreResult<String> {
    let statuses = crate::prompt_gen::check_all_prompt_statuses(layout)?;
    if statuses.is_empty() {
        return Ok("No prompts to check.".to_string());
    }
    let mut output = String::new();
    for s in &statuses {
        let icon = if s.is_stale { "⚠" } else { "✓" };
        let status = if s.is_stale { "stale" } else { "up-to-date" };
        output.push_str(&format!("{icon} {}: {status}\n", s.filename));
        for r in &s.reasons {
            output.push_str(&format!("    {r}\n"));
        }
    }
    Ok(output)
}

fn execute_regenerate_prompts(layout: &RepoLayout) -> CoreResult<String> {
    let results = crate::prompt_gen::regenerate_stale(layout, false)?;
    if results.is_empty() {
        return Ok("All prompts are up-to-date.".to_string());
    }
    let mut output = format!("Regenerated {} prompt(s):\n", results.len());
    for r in &results {
        output.push_str(&format!("  - {}\n", r.artifact.path));
    }
    Ok(output)
}

// ---------------------------------------------------------------------------
// Session summary
// ---------------------------------------------------------------------------

/// Build a concise summary of artifacts created during this session.
pub fn build_session_summary(ctx: &ChatContext) -> String {
    if ctx.artifacts.is_empty() {
        return String::new();
    }

    let mut summary = String::from("Artifacts created in this session:\n");
    for artifact in &ctx.artifacts {
        summary.push_str(&format!(
            "- {}: \"{}\" ({}) — {}\n",
            artifact.kind, artifact.id, artifact.path, artifact.summary
        ));
    }
    summary
}

// ---------------------------------------------------------------------------
// REPL loop
// ---------------------------------------------------------------------------

/// Check if the user wants to exit.
pub fn is_exit(input: &str) -> bool {
    matches!(
        input.trim().to_lowercase().as_str(),
        "exit" | "quit" | "bye" | "/exit" | "/quit"
    )
}

/// Find the most recent chat session that can be resumed.
fn find_resumable_session(layout: &RepoLayout) -> Option<ConversationSession> {
    let sessions = list_sessions(&layout.conversations_dir()).ok()?;
    sessions
        .into_iter()
        .rev() // most recent first (list_sessions sorts by started_at)
        .find(|s| s.workflow == WorkflowKind::Chat)
}

/// Reconstruct ChatContext from a saved session's steps.
fn restore_context_from_session(session: &ConversationSession) -> ChatContext {
    let mut ctx = ChatContext::new();

    for step in &session.steps {
        match step.step_type {
            StepType::UserInput => {
                ctx.history.push(ChatMessage {
                    role: MessageRole::User,
                    content: step.content.clone(),
                });
            }
            StepType::Info => {
                ctx.history.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: step.content.clone(),
                });
            }
            StepType::Write => {
                // Write steps are action summaries like "Created contract: kv-store (...)"
                // Parse them back into SessionArtifacts for the session summary
                if let Some(artifact) = parse_artifact_from_summary(&step.content) {
                    ctx.artifacts.push(artifact);
                }
            }
            _ => {}
        }
    }

    ctx
}

/// Try to parse a SessionArtifact from a saved action summary string.
fn parse_artifact_from_summary(summary: &str) -> Option<SessionArtifact> {
    // Summaries look like "Created contract: kv-store (specs/contracts/kv-store.toml)"
    let lower = summary.to_lowercase();
    let (kind, prefix) = if lower.starts_with("created contract") || lower.starts_with("updated contract") {
        (ArtifactCategory::Contract, "contract")
    } else if lower.starts_with("created conformance") || lower.starts_with("generated conformance") {
        (ArtifactCategory::Conformance, "conformance")
    } else if lower.starts_with("created behavior") || lower.starts_with("generated behavior") {
        (ArtifactCategory::Behavior, "behavior")
    } else if lower.starts_with("created gate") {
        (ArtifactCategory::Gate, "gate")
    } else if lower.contains("prompt") {
        (ArtifactCategory::Prompt, "prompt")
    } else {
        return None;
    };

    // Extract ID: text after ": " and before " ("
    let id = summary
        .split(": ")
        .nth(1)
        .and_then(|s| s.split(" (").next())
        .unwrap_or(prefix)
        .to_string();

    // Extract path: text inside parentheses
    let path = summary
        .split('(')
        .nth(1)
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or("")
        .to_string();

    Some(SessionArtifact {
        kind,
        id,
        path,
        summary: summary.to_string(),
    })
}

/// Run the interactive chat design session.
pub fn run_chat(
    layout: &RepoLayout,
    driver: &dyn ConversationDriver,
) -> CoreResult<()> {
    let ai_provider = build_ai_provider(layout)?;
    let (repo_context, _warnings) = load_context(layout);

    let heading_style = Style::new().bold().cyan();
    let dim_style = Style::new().dim();
    let success_style = Style::new().green().bold();
    let action_style = Style::new().yellow();

    // Check for a resumable session
    let (mut ctx, mut session) = if let Some(prev) = find_resumable_session(layout) {
        let age = chrono::Utc::now() - prev.started_at;
        let age_str = if age.num_days() > 0 {
            format!("{}d ago", age.num_days())
        } else if age.num_hours() > 0 {
            format!("{}h ago", age.num_hours())
        } else {
            format!("{}m ago", age.num_minutes())
        };

        let step_count = prev.steps.iter().filter(|s| s.step_type == StepType::UserInput).count();
        let prompt = format!(
            "  Resume previous session ({step_count} turns, {age_str})? [Y/n] "
        );

        let answer = driver
            .present_question(&Question::simple(&prompt))
            .unwrap_or_default();

        if answer.trim().is_empty() || answer.trim().to_lowercase().starts_with('y') {
            let ctx = restore_context_from_session(&prev);

            // Reopen the session (create a continuation)
            let mut session = ConversationSession::new(WorkflowKind::Chat);
            // Copy over previous steps so the full history is preserved on next save
            session.steps = prev.steps;

            println!("\n{}", heading_style.apply_to("  Lexicon Design Session (resumed)"));
            println!("{}", dim_style.apply_to("  ─".to_string() + &"─".repeat(58)));

            // Show a brief recap of what was discussed
            let user_turns: Vec<_> = ctx.history.iter().filter(|m| m.role == MessageRole::User).collect();
            if !user_turns.is_empty() {
                println!("  {} previous turns restored. Last topic:", dim_style.apply_to(format!("{}", user_turns.len())));
                if let Some(last) = user_turns.last() {
                    let preview: String = last.content.chars().take(80).collect();
                    println!("  {}", dim_style.apply_to(format!("  \"{preview}...\"")));
                }
            }
            if !ctx.artifacts.is_empty() {
                println!("  {} artifact(s) in session.", dim_style.apply_to(format!("{}", ctx.artifacts.len())));
            }
            println!("  Type {} to end the session.\n", dim_style.apply_to("'exit'"));

            (ctx, session)
        } else {
            (ChatContext::new(), ConversationSession::new(WorkflowKind::Chat))
        }
    } else {
        (ChatContext::new(), ConversationSession::new(WorkflowKind::Chat))
    };

    // Show header for new sessions
    if session.steps.is_empty() {
        println!("\n{}", heading_style.apply_to("  Lexicon Design Session"));
        println!("{}", dim_style.apply_to("  ─".to_string() + &"─".repeat(58)));
        println!("  Describe what you want to build. I'll help you design the");
        println!("  contracts, gates, and constraints, then generate an");
        println!("  implementation prompt.");
        println!("  Type {} to end the session.\n", dim_style.apply_to("'exit'"));
    }

    loop {
        let input = match driver.present_question(&Question::simple("you> ")) {
            Ok(input) => input,
            Err(_) => break,
        };

        if is_exit(&input) {
            break;
        }

        if input.trim().is_empty() {
            continue;
        }

        session.add_step(StepType::UserInput, input.clone());
        ctx.history.push(ChatMessage {
            role: MessageRole::User,
            content: input.clone(),
        });

        // Build AI message with full context
        let session_summary = build_session_summary(&ctx);
        let history_pairs: Vec<(String, String)> = ctx
            .history
            .iter()
            .rev()
            .take(MAX_HISTORY_TURNS * 2) // pairs of user+assistant
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| (m.role.to_string(), m.content.clone()))
            .collect();

        let user_msg =
            build_chat_user_message(&repo_context, &session_summary, &history_pairs, &input);

        // Get AI response with spinner
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.set_message("Thinking...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let raw_response = match ai_provider.complete(CHAT_SYSTEM, &user_msg) {
            Ok(r) => {
                spinner.finish_and_clear();
                r
            }
            Err(e) => {
                spinner.finish_and_clear();
                let error_style = Style::new().red().bold();
                println!("  {} AI error: {e}", error_style.apply_to("✗"));
                continue;
            }
        };

        // Parse response for action directives
        let parsed = parse_ai_response(&raw_response);

        // Execute any actions
        for action in &parsed.actions {
            println!(
                "  {} {}",
                action_style.apply_to("▸"),
                action_style.apply_to(action.label())
            );
            match execute_action(layout, action, &mut ctx, ai_provider.as_ref()) {
                Ok(summary) => {
                    println!("  {} {}", success_style.apply_to("✓"), summary);
                    session.add_step(StepType::Write, summary);
                }
                Err(e) => {
                    let error_style = Style::new().red().bold();
                    println!("  {} Action failed: {e}", error_style.apply_to("✗"));
                }
            }
        }

        // Display the conversational part
        if !parsed.display_text.is_empty() {
            ctx.history.push(ChatMessage {
                role: MessageRole::Assistant,
                content: parsed.display_text.clone(),
            });
            session.add_step(StepType::Info, parsed.display_text.clone());
            println!();
            println!("{}", parsed.display_text);
            println!();
        }

        // Auto-save after each turn so interrupted sessions can be resumed
        let _ = save_session(&layout.conversations_dir(), &session);
    }

    session.complete(None);
    let _ = save_session(&layout.conversations_dir(), &session);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_exit() {
        assert!(is_exit("exit"));
        assert!(is_exit("quit"));
        assert!(is_exit("bye"));
        assert!(is_exit("  Exit  "));
        assert!(is_exit("/exit"));
        assert!(is_exit("/quit"));
        assert!(!is_exit("help"));
        assert!(!is_exit("create contract"));
    }

    #[test]
    fn test_parse_ai_response_no_actions() {
        let response = "Here's what I suggest for your cache contract.\n\nYou should consider adding TTL support.";
        let parsed = parse_ai_response(response);
        assert!(parsed.actions.is_empty());
        assert_eq!(
            parsed.display_text,
            "Here's what I suggest for your cache contract.\n\nYou should consider adding TTL support."
        );
    }

    #[test]
    fn test_parse_ai_response_create_contract() {
        let response = r#"I'll create a contract for you.

:::ACTION CREATE_CONTRACT
id = "kv-store"
title = "Key-Value Store"
scope = "Core storage API"
status = "draft"
stability = "experimental"
version = "0.1.0"
:::

The contract has been created with the basic structure."#;

        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        match &parsed.actions[0] {
            ChatAction::CreateContract { toml_content } => {
                assert!(toml_content.contains("kv-store"));
                assert!(toml_content.contains("Key-Value Store"));
            }
            _ => panic!("Expected CreateContract"),
        }
        assert!(parsed.display_text.contains("I'll create a contract"));
        assert!(parsed.display_text.contains("The contract has been created"));
        assert!(!parsed.display_text.contains(":::"));
    }

    #[test]
    fn test_parse_ai_response_multiple_actions() {
        let response = r#"Creating artifacts.

:::ACTION CREATE_CONTRACT
id = "cache"
title = "Cache"
scope = "Caching"
status = "draft"
stability = "experimental"
version = "0.1.0"
:::

Now let me add gates.

:::ACTION CREATE_GATE
[[gates]]
id = "fmt"
label = "Formatting"
command = "cargo fmt --check"
category = "required"
:::

Done!"#;

        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 2);
        assert!(matches!(&parsed.actions[0], ChatAction::CreateContract { .. }));
        assert!(matches!(&parsed.actions[1], ChatAction::CreateGate { .. }));
        assert!(parsed.display_text.contains("Creating artifacts."));
        assert!(parsed.display_text.contains("Now let me add gates."));
        assert!(parsed.display_text.contains("Done!"));
    }

    #[test]
    fn test_parse_ai_response_update_contract() {
        let response = r#":::ACTION UPDATE_CONTRACT kv-store
id = "kv-store"
title = "Key-Value Store"
scope = "Updated scope"
status = "draft"
stability = "experimental"
version = "0.1.0"
:::"#;

        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        match &parsed.actions[0] {
            ChatAction::UpdateContract { id, toml_content } => {
                assert_eq!(id, "kv-store");
                assert!(toml_content.contains("Updated scope"));
            }
            _ => panic!("Expected UpdateContract"),
        }
    }

    #[test]
    fn test_parse_ai_response_generate_prompt() {
        let response = "Your specification looks comprehensive.\n\n:::ACTION GENERATE_PROMPT\n:::\n\nThe prompt has been compiled.";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        assert!(matches!(&parsed.actions[0], ChatAction::GeneratePrompt));
    }

    #[test]
    fn test_parse_ai_response_run_verify() {
        let response = "Let me check the current state.\n\n:::ACTION RUN_VERIFY\n:::";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        assert!(matches!(&parsed.actions[0], ChatAction::RunVerify));
    }

    #[test]
    fn test_parse_ai_response_strips_directives() {
        let response = "Before\n:::ACTION GENERATE_PROMPT\n:::\nAfter";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.display_text, "Before\nAfter");
    }

    #[test]
    fn test_build_session_summary_empty() {
        let ctx = ChatContext::new();
        assert_eq!(build_session_summary(&ctx), "");
    }

    #[test]
    fn test_build_session_summary_with_artifacts() {
        let mut ctx = ChatContext::new();
        ctx.artifacts.push(SessionArtifact {
            kind: ArtifactCategory::Contract,
            id: "kv-store".to_string(),
            path: "specs/contracts/kv-store.toml".to_string(),
            summary: "2 invariants, 1 required".to_string(),
        });
        ctx.artifacts.push(SessionArtifact {
            kind: ArtifactCategory::Gate,
            id: "gates".to_string(),
            path: "specs/gates.toml".to_string(),
            summary: "3 gates defined".to_string(),
        });

        let summary = build_session_summary(&ctx);
        assert!(summary.contains("kv-store"));
        assert!(summary.contains("Gate"));
        assert!(summary.contains("3 gates defined"));
    }

    #[test]
    fn test_parse_directive_unknown() {
        assert!(parse_directive("UNKNOWN_ACTION", "content").is_none());
    }

    #[test]
    fn test_parse_ai_response_conformance() {
        let response = ":::ACTION CREATE_CONFORMANCE kv-store\n:::";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        match &parsed.actions[0] {
            ChatAction::CreateConformance { contract_id } => {
                assert_eq!(contract_id, "kv-store");
            }
            _ => panic!("Expected CreateConformance"),
        }
    }

    #[test]
    fn test_parse_new_directives() {
        let cases = vec![
            (":::ACTION CREATE_PROPERTY_TESTS kv-store\n:::", "kv-store", "CreatePropertyTests"),
            (":::ACTION CREATE_FUZZ_TARGET cache\n:::", "cache", "CreateFuzzTarget"),
            (":::ACTION CREATE_EDGE_CASES auth\n:::", "auth", "CreateEdgeCases"),
        ];

        for (input, expected_id, label) in cases {
            let parsed = parse_ai_response(input);
            assert_eq!(parsed.actions.len(), 1, "Failed for {label}");
            let contract_id = match &parsed.actions[0] {
                ChatAction::CreatePropertyTests { contract_id } => contract_id,
                ChatAction::CreateFuzzTarget { contract_id } => contract_id,
                ChatAction::CreateEdgeCases { contract_id } => contract_id,
                other => panic!("Expected {label}, got {other:?}"),
            };
            assert_eq!(contract_id, expected_id, "Wrong ID for {label}");
        }
    }

    #[test]
    fn test_parse_infer_contract() {
        let response = ":::ACTION INFER_CONTRACT\n:::";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        assert!(matches!(&parsed.actions[0], ChatAction::InferContract));
    }

    #[test]
    fn test_parse_coverage_report() {
        let response = ":::ACTION COVERAGE_REPORT\n:::";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        assert!(matches!(&parsed.actions[0], ChatAction::CoverageReport));
    }

    #[test]
    fn test_parse_api_scan() {
        let response = ":::ACTION API_SCAN\n:::";
        let parsed = parse_ai_response(response);
        assert_eq!(parsed.actions.len(), 1);
        assert!(matches!(&parsed.actions[0], ChatAction::ApiScan));
    }
}
