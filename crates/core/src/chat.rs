//! AI-guided architecture design session.
//!
//! Provides a conversational REPL where an AI agent drives the user through
//! designing contracts, gates, conformance tests, and other artifacts, then
//! compiles everything into a constraint-aware implementation prompt.

use lexicon_ai::boundary::AiProvider;
use lexicon_ai::error::AiError;
use lexicon_ai::generate::load_context_selective;
use lexicon_ai::prompt::{build_chat_user_message, estimate_tokens, CHAT_SCHEMAS, CHAT_SYSTEM};
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
const MAX_HISTORY_TURNS: usize = 3;

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

/// Summarize an AI response for compact history storage.
///
/// Keeps the first 2-3 sentences, replaces code/TOML blocks with a short
/// placeholder, and caps total length. The full text is still printed to
/// the console and saved in the session file.
fn summarize_for_history(text: &str) -> String {
    const MAX_CHARS: usize = 300;

    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_block_replaced = false;

    for line in text.lines() {
        if line.trim_start().starts_with("```") || line.trim_start().starts_with(":::") {
            if in_code_block {
                in_code_block = false;
                continue;
            }
            in_code_block = true;
            if !code_block_replaced {
                result.push_str("[code omitted] ");
                code_block_replaced = true;
            }
            continue;
        }
        if in_code_block {
            continue;
        }
        // Skip blank lines to keep it compact
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(trimmed);
        if result.len() >= MAX_CHARS {
            break;
        }
    }

    if result.len() > MAX_CHARS {
        result.truncate(MAX_CHARS);
        result.push_str("...");
    }
    result
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
    // Prefer contracts created during this session, but fall back to all on-disk contracts
    let mut contract_ids: Vec<String> = ctx
        .artifacts
        .iter()
        .filter(|a| a.kind == ArtifactCategory::Contract)
        .map(|a| a.id.clone())
        .collect();

    if contract_ids.is_empty() {
        contract_ids = list_contract_ids(layout);
    }

    if contract_ids.is_empty() {
        return Ok("No contracts found. Create a contract first.".to_string());
    }

    let mut successes = Vec::new();
    let mut failures = Vec::new();
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
                successes.push(format!("Generated prompt at {path}"));
            }
            Err(e) => {
                failures.push(format!("Failed to generate prompt for {contract_id}: {e}"));
            }
        }
    }

    if failures.is_empty() {
        Ok(successes.join("\n"))
    } else if successes.is_empty() {
        // All failed — return as error so it gets fed back to the AI
        Err(CoreError::Other(failures.join("\n")))
    } else {
        // Mixed results — report successes but also return failures as error
        let summary = successes.join("\n");
        // Print successes directly since we'll return the failures as an error
        println!("  {summary}");
        Err(CoreError::Other(failures.join("\n")))
    }
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

/// List contract IDs available on disk.
fn list_contract_ids(layout: &RepoLayout) -> Vec<String> {
    let dir = layout.contracts_dir();
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "toml") {
                if let Some(stem) = entry.path().file_stem() {
                    ids.push(stem.to_string_lossy().to_string());
                }
            }
        }
    }
    ids.sort();
    ids
}

/// Load a single contract by ID from the contracts directory.
fn load_contract_by_id(layout: &RepoLayout, contract_id: &str) -> CoreResult<Contract> {
    let path = layout.contracts_dir().join(format!("{contract_id}.toml"));
    if !path.exists() {
        let available = list_contract_ids(layout);
        let hint = if available.is_empty() {
            "No contracts exist yet — create one first.".to_string()
        } else {
            format!("Available contracts: {}", available.join(", "))
        };
        return Err(CoreError::Other(format!(
            "Contract not found: {contract_id}. {hint}"
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
// Slash commands
// ---------------------------------------------------------------------------

/// Available AI models for the `/models` command.
const AVAILABLE_MODELS: &[(&str, &str)] = &[
    ("claude-haiku-4-5-20251001", "Haiku 4.5 (fast, cheapest)"),
    ("claude-sonnet-4-20250514", "Sonnet 4 (balanced)"),
    ("claude-opus-4-20250514", "Opus 4 (most capable)"),
];

/// Dispatch a slash command entered by the user.
fn handle_slash_command(
    cmd: &str,
    layout: &RepoLayout,
    ai_provider: &mut Box<dyn AiProvider>,
    ctx: &ChatContext,
    dim_style: &Style,
) -> CoreResult<()> {
    match cmd.trim().to_lowercase().as_str() {
        "help" | "h" => show_help(dim_style),
        "models" | "model" => select_model(layout, ai_provider, dim_style)?,
        "status" | "s" => show_status(ai_provider, ctx, dim_style),
        "artifacts" | "a" => show_artifacts(ctx, dim_style),
        "verify" | "v" => run_verify_inline(layout, dim_style)?,
        other => {
            println!(
                "  Unknown command: /{other}. Type /help for available commands."
            );
        }
    }
    Ok(())
}

/// Display available slash commands.
fn show_help(dim_style: &Style) {
    println!();
    println!("  Available commands:");
    println!("    {}       Show this help message", dim_style.apply_to("/help, /h"));
    println!("    {}     Show session status (model, turns, artifacts)", dim_style.apply_to("/status, /s"));
    println!("    {}  List artifacts created this session", dim_style.apply_to("/artifacts, /a"));
    println!("    {}        List and select an AI model", dim_style.apply_to("/models"));
    println!("    {}     Run verification (gates + scoring)", dim_style.apply_to("/verify, /v"));
    println!("    {}   End the session", dim_style.apply_to("/exit, /quit"));
    println!();
}

/// Show current session status.
fn show_status(
    ai_provider: &Box<dyn AiProvider>,
    ctx: &ChatContext,
    dim_style: &Style,
) {
    let model = ai_provider.model_id();
    let user_turns = ctx.history.iter().filter(|m| m.role == MessageRole::User).count();
    let artifact_count = ctx.artifacts.len();

    println!();
    println!("  Session status:");
    println!("    Model:     {model}");
    println!("    Turns:     {user_turns}");
    println!("    Artifacts: {artifact_count}");
    if artifact_count > 0 {
        for a in &ctx.artifacts {
            println!("      {} {} {}", dim_style.apply_to("•"), a.kind, dim_style.apply_to(&a.id));
        }
    }
    println!();
}

/// List all artifacts created during this session.
fn show_artifacts(ctx: &ChatContext, dim_style: &Style) {
    println!();
    if ctx.artifacts.is_empty() {
        println!("  No artifacts created yet.");
    } else {
        println!("  Artifacts ({}):", ctx.artifacts.len());
        for a in &ctx.artifacts {
            println!(
                "    {} {} {} {}",
                dim_style.apply_to("•"),
                a.kind,
                a.id,
                dim_style.apply_to(format!("({})", a.path)),
            );
            if !a.summary.is_empty() {
                println!("      {}", dim_style.apply_to(&a.summary));
            }
        }
    }
    println!();
}

/// Run verification inline and display a summary.
fn run_verify_inline(layout: &RepoLayout, dim_style: &Style) -> CoreResult<()> {
    println!();
    println!("  Running verification...");

    let result = crate::verify::verify(layout)?;

    // Gates
    if result.gate_results.is_empty() {
        println!("  {}", dim_style.apply_to("No gates configured."));
    } else {
        let passed = result.gate_results.iter().filter(|g| g.passed()).count();
        let total = result.gate_results.len();
        let gate_style = if passed == total {
            Style::new().green().bold()
        } else {
            Style::new().red().bold()
        };
        println!("  Gates: {}", gate_style.apply_to(format!("{passed}/{total} passed")));
        for g in &result.gate_results {
            let icon = if g.passed() { "✓" } else { "✗" };
            let style = if g.passed() {
                Style::new().green()
            } else {
                Style::new().red()
            };
            println!("    {} {}", style.apply_to(icon), g.gate_id);
        }
    }

    // Scoring
    if let Some(ref score) = result.score_report {
        let pct = (score.total_score * 100.0).round() as u32;
        let verdict_style = match score.verdict {
            lexicon_scoring::engine::Verdict::Pass => Style::new().green().bold(),
            lexicon_scoring::engine::Verdict::Warn => Style::new().yellow().bold(),
            lexicon_scoring::engine::Verdict::Fail => Style::new().red().bold(),
        };
        println!(
            "  Score: {} ({:?})",
            verdict_style.apply_to(format!("{pct}%")),
            score.verdict,
        );
    }

    // Coverage
    if let Some(ref cov) = result.coverage_report {
        if cov.total_clauses > 0 {
            println!(
                "  Coverage: {}/{} clauses covered",
                cov.total_covered, cov.total_clauses,
            );
        }
    }

    // Prompt warnings
    if !result.prompt_warnings.is_empty() {
        println!("  Warnings: {}", result.prompt_warnings.len());
    }

    println!();
    Ok(())
}

/// Present a model selection menu and swap the AI provider if the user picks one.
fn select_model(
    layout: &RepoLayout,
    ai_provider: &mut Box<dyn AiProvider>,
    dim_style: &Style,
) -> CoreResult<()> {
    let current = ai_provider.model_id();

    println!();
    println!("  Available models:");
    for (i, (id, desc)) in AVAILABLE_MODELS.iter().enumerate() {
        let marker = if *id == current { " (current)" } else { "" };
        println!("    {}. {desc}{marker}", i + 1);
    }
    println!();
    print!("  Select model [1-{}] or Enter to cancel: ", AVAILABLE_MODELS.len());
    // Flush so the prompt appears before reading
    use std::io::Write;
    std::io::stdout().flush().ok();

    let mut choice = String::new();
    std::io::stdin()
        .read_line(&mut choice)
        .map_err(|e| CoreError::Other(format!("Failed to read input: {e}")))?;

    let choice = choice.trim();
    if choice.is_empty() {
        println!("  {}", dim_style.apply_to("Cancelled."));
        return Ok(());
    }

    match choice.parse::<usize>() {
        Ok(n) if n >= 1 && n <= AVAILABLE_MODELS.len() => {
            let (id, desc) = AVAILABLE_MODELS[n - 1];
            *ai_provider = build_ai_provider(layout, Some(id))?;
            let success_style = Style::new().green().bold();
            println!("  {} Switched to {desc} ({id})", success_style.apply_to("✓"));
        }
        _ => {
            println!("  Invalid selection.");
        }
    }
    Ok(())
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

/// Returns true if a message is substantive (not just a short affirmative reply).
fn is_substantive(msg: &str) -> bool {
    let trimmed = msg.trim().to_lowercase();
    if trimmed.len() <= 5 {
        return false;
    }
    let trivial = ["yes", "no", "ok", "okay", "sure", "thanks", "thank you", "yep", "nope", "y", "n"];
    !trivial.contains(&trimmed.as_str())
}

/// Truncate a string to `max` characters, appending "..." if truncated.
fn truncate_with_ellipsis(s: &str, max: usize) -> String {
    // Take just the first line for display
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.chars().count() <= max {
        first_line.to_string()
    } else {
        let truncated: String = first_line.chars().take(max).collect();
        format!("{truncated}...")
    }
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
                    content: summarize_for_history(&step.content),
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
    model: Option<&str>,
) -> CoreResult<()> {
    let mut ai_provider = build_ai_provider(layout, model)?;

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
            .present_question(&Question::simple(&prompt).with_default("y"))
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
            let turn_count = user_turns.len();
            if turn_count > 0 {
                println!("  {} previous turns restored.", dim_style.apply_to(format!("{turn_count}")));
            }

            // Topic: first substantive user message (the opening request)
            if let Some(topic_msg) = ctx.history.iter()
                .filter(|m| m.role == MessageRole::User && is_substantive(&m.content))
                .next()
            {
                let preview = truncate_with_ellipsis(&topic_msg.content, 72);
                println!("  Topic: {}", dim_style.apply_to(format!("\"{preview}\"")));
            }

            // Last: most recent assistant message (where we left off)
            if let Some(last_asst) = ctx.history.iter()
                .rev()
                .find(|m| m.role == MessageRole::Assistant)
            {
                let preview = truncate_with_ellipsis(&last_asst.content, 72);
                println!("  Last:  {}", dim_style.apply_to(format!("\"{preview}\"")));
            }

            if !ctx.artifacts.is_empty() {
                println!("  {} artifact(s) in session.", dim_style.apply_to(format!("{}", ctx.artifacts.len())));
            }
            println!(
                "  Type {} for commands or {} to end.\n",
                dim_style.apply_to("/help"),
                dim_style.apply_to("'exit'"),
            );

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
        println!(
            "  Type {} for commands or {} to end.\n",
            dim_style.apply_to("/help"),
            dim_style.apply_to("'exit'"),
        );
    }

    let mut auto_followup = false;

    // Set up rustyline editor with history for the chat REPL
    let rl_config = rustyline::Config::builder()
        .max_history_size(500)
        .expect("valid history size")
        .auto_add_history(true)
        .build();
    let mut rl = rustyline::DefaultEditor::with_config(rl_config)
        .unwrap_or_else(|_| rustyline::DefaultEditor::new().expect("rustyline editor"));

    // Load history from a file in the .lexicon directory if it exists
    let history_path = layout.root.join(".lexicon").join("chat_history");
    let _ = rl.load_history(&history_path);

    'chat: loop {
        // If auto-following up after errors, skip user input
        if !auto_followup {
            let input = match rl.readline("you> ") {
                Ok(line) => line,
                Err(rustyline::error::ReadlineError::Interrupted | rustyline::error::ReadlineError::Eof) => break,
                Err(_) => break,
            };

            if is_exit(&input) {
                break;
            }

            if input.trim().is_empty() {
                continue;
            }

            // Intercept slash commands before adding to history/AI context
            if let Some(cmd) = input.trim().strip_prefix('/') {
                handle_slash_command(cmd, layout, &mut ai_provider, &ctx, &dim_style)?;
                continue;
            }

            session.add_step(StepType::UserInput, input.clone());
            ctx.history.push(ChatMessage {
                role: MessageRole::User,
                content: input.clone(),
            });
        }
        auto_followup = false;

        // Build AI message with full context
        let session_summary = build_session_summary(&ctx);

        // Extract current user message (the last one) — sent separately, not in history
        let current_input = ctx
            .history
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        // Build history excluding the current user message.
        // Count actual turns (user+assistant pairs), not individual messages.
        let history_without_current: Vec<_> = {
            let len = ctx.history.len();
            // Skip the trailing user message (it's the current input)
            let end = if ctx.history.last().is_some_and(|m| m.role == MessageRole::User) {
                len.saturating_sub(1)
            } else {
                len
            };
            let msgs = &ctx.history[..end];
            // Count turns by counting user messages, take last MAX_HISTORY_TURNS turns
            let mut turn_count = 0;
            let mut start = msgs.len();
            for (i, m) in msgs.iter().enumerate().rev() {
                if m.role == MessageRole::User {
                    turn_count += 1;
                    if turn_count > MAX_HISTORY_TURNS {
                        break;
                    }
                    start = i;
                }
            }
            msgs[start..].to_vec()
        };

        let history_pairs: Vec<(String, String)> = history_without_current
            .iter()
            .map(|m| (m.role.to_string(), m.content.clone()))
            .collect();

        // Include TOML schemas on first turn or when user mentions creating/updating artifacts
        let is_first_turn = history_without_current.is_empty();
        let input_lower = current_input.to_lowercase();
        let mentions_artifact = input_lower.contains("contract")
            || input_lower.contains("gate")
            || input_lower.contains("create")
            || input_lower.contains("update");
        let include_schemas = is_first_turn || mentions_artifact;

        // Build repo context with only active contract IDs for full detail
        let active_ids: Vec<&str> = ctx
            .artifacts
            .iter()
            .filter(|a| a.kind == ArtifactCategory::Contract)
            .map(|a| a.id.as_str())
            .collect();
        let (repo_context, _warnings) = load_context_selective(layout, &active_ids);

        let user_msg = build_chat_user_message(
            &repo_context,
            &session_summary,
            &history_pairs,
            &current_input,
            include_schemas,
        );

        // Estimate token usage and warn if high
        let system_tokens = estimate_tokens(CHAT_SYSTEM);
        let user_tokens = estimate_tokens(&user_msg);
        let total_tokens = system_tokens + user_tokens;
        if total_tokens > 15_000 {
            let warn_style = Style::new().yellow();
            eprintln!(
                "  {} High token estimate: ~{} (system: ~{}, user: ~{})",
                warn_style.apply_to("⚠"),
                total_tokens,
                system_tokens,
                user_tokens
            );
        }

        // Get AI response with spinner and retry on transient failures
        const MAX_RETRIES: u32 = 3;
        let raw_response = 'retry: {
            let mut last_err = String::new();
            for attempt in 0..=MAX_RETRIES {
                let spinner = ProgressBar::new_spinner();
                spinner.set_style(
                    ProgressStyle::with_template("{spinner:.cyan} {msg}")
                        .unwrap()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
                );
                if attempt == 0 {
                    spinner.set_message("Thinking...");
                } else {
                    spinner.set_message(format!("Retrying ({attempt}/{MAX_RETRIES})..."));
                }
                spinner.enable_steady_tick(std::time::Duration::from_millis(80));

                match ai_provider.complete(CHAT_SYSTEM, &user_msg) {
                    Ok(r) => {
                        spinner.finish_and_clear();
                        break 'retry r;
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
                        last_err = format!("{e}");
                        if attempt < MAX_RETRIES {
                            let (delay, label) = match &e {
                                AiError::RateLimited { retry_after_secs, .. } => {
                                    let secs = retry_after_secs.unwrap_or(30);
                                    (std::time::Duration::from_secs(secs), "Rate limited")
                                }
                                _ => {
                                    (std::time::Duration::from_secs(2u64.pow(attempt)), "Network error")
                                }
                            };
                            let warn_style = Style::new().yellow();
                            println!(
                                "  {} {label}, retrying in {}s...",
                                warn_style.apply_to("⚠"),
                                delay.as_secs()
                            );
                            std::thread::sleep(delay);
                        }
                    }
                }
            }
            // All retries exhausted
            let error_style = Style::new().red().bold();
            println!(
                "  {} AI error after {} retries: {last_err}",
                error_style.apply_to("✗"),
                MAX_RETRIES
            );
            // Remove the user message from history since we got no response
            if ctx.history.last().is_some_and(|m| m.role == MessageRole::User) {
                ctx.history.pop();
            }
            session.steps.pop(); // remove the UserInput step
            continue 'chat;
        };

        // Parse response for action directives
        let parsed = parse_ai_response(&raw_response);

        // Execute any actions, collecting errors to feed back to the AI
        let mut action_errors: Vec<String> = Vec::new();
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
                    let error_msg = format!("{} failed: {e}", action.label());
                    println!("  {} {error_msg}", error_style.apply_to("✗"));
                    action_errors.push(error_msg);
                }
            }
        }

        // If actions failed, suppress the AI's text (it may falsely claim success)
        // and inject error context so the AI can suggest fixes on the next turn.
        if !action_errors.is_empty() {
            // If errors mention specific contracts, include their raw content
            // so the AI can emit corrected UPDATE_CONTRACT directives.
            let mut contract_contents = String::new();
            for err in &action_errors {
                // Extract contract IDs from error messages like "...for contract_id: ..."
                for word in err.split_whitespace() {
                    let candidate = word.trim_matches(|c: char| c == ':' || c == ',' || c == '.');
                    let contract_path = layout.contracts_dir().join(format!("{candidate}.toml"));
                    if contract_path.is_file() {
                        if let Ok(content) = std::fs::read_to_string(&contract_path) {
                            // Truncate to first 20 lines to avoid bloating token usage
                            let truncated: String = content
                                .lines()
                                .take(20)
                                .collect::<Vec<_>>()
                                .join("\n");
                            let suffix = if content.lines().count() > 20 {
                                let remaining = content.lines().count() - 20;
                                format!("\n[truncated — {remaining} more lines]")
                            } else {
                                String::new()
                            };
                            contract_contents.push_str(&format!(
                                "\n--- Content of {candidate}.toml (truncated) ---\n{truncated}{suffix}\n"
                            ));
                        }
                    }
                }
            }
            // Cap total error feedback to prevent token blowup
            const MAX_ERROR_FEEDBACK_CHARS: usize = 2000;
            if contract_contents.len() > MAX_ERROR_FEEDBACK_CHARS {
                contract_contents.truncate(MAX_ERROR_FEEDBACK_CHARS);
                contract_contents.push_str("\n[contract content truncated for brevity]");
            }

            let error_feedback = format!(
                "[SYSTEM: The following actions failed. Your previous response text was \
                suppressed because it contained false claims about files being created.\n\
                \n\
                INSTRUCTIONS: You MUST fix the errors by emitting :::ACTION directives \
                (e.g., UPDATE_CONTRACT <id> with corrected TOML to fix malformed contracts). \
                Do NOT just describe what you would do — actually emit the directives. \
                Do NOT claim any files were created unless you emit the directive in this \
                response.\n\
                \n\
                IMPORTANT: All top-level fields are required. Use the schema below.]\n\n\
                {}\n{contract_contents}\n\n{CHAT_SCHEMAS}",
                action_errors.join("\n")
            );
            ctx.history.push(ChatMessage {
                role: MessageRole::User,
                content: error_feedback,
            });
            println!();
            let warn_style = Style::new().yellow();
            println!(
                "  {} Errors occurred — asking AI to diagnose and suggest fixes...",
                warn_style.apply_to("⚠")
            );
            println!();
            auto_followup = true;
        } else {
            // Display the conversational part only when no actions failed
            if !parsed.display_text.is_empty() {
                ctx.history.push(ChatMessage {
                    role: MessageRole::Assistant,
                    content: summarize_for_history(&parsed.display_text),
                });
                session.add_step(StepType::Info, parsed.display_text.clone());
                println!();
                println!("{}", parsed.display_text);
                println!();
            }
        }

        // Auto-save after each turn so interrupted sessions can be resumed
        let _ = save_session(&layout.conversations_dir(), &session);
    }

    // Save readline history
    let _ = rl.save_history(&history_path);

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
