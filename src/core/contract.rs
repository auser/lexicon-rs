use crate::audit::writer::write_audit_record;
use crate::conversation::driver::ConversationDriver;
use crate::conversation::engine::ConversationEngine;
use crate::conversation::session::save_session;
use crate::conversation::workflow::{
    Proposal, Question, StepInput, StepOutput, Workflow, WorkflowStep,
};
use crate::repo::layout::RepoLayout;
use crate::spec::audit::AuditRecord;
use crate::spec::common::{Actor, AuditAction, Severity, WorkflowKind};
use crate::spec::contract::{Contract, Invariant, Semantic};
use crate::spec::validation::slugify;

use crate::core::error::CoreResult;

struct ContractNewContext {
    title: String,
    description: String,
    scope: String,
    invariants: Vec<String>,
    required: Vec<String>,
    forbidden: Vec<String>,
    draft: Option<Contract>,
}

struct ContractNewWorkflow {
    steps: Vec<WorkflowStep>,
    layout: RepoLayout,
}

impl ContractNewWorkflow {
    fn new(layout: RepoLayout) -> Self {
        Self {
            steps: vec![
                WorkflowStep::new("title", "Contract title (e.g. Key-Value Store)?"),
                WorkflowStep::new("description", "Describe the contract's purpose (leave empty for AI-assisted)").skippable(),
                WorkflowStep::new("scope", "What scope does this contract cover? (leave empty for AI-assisted)").skippable(),
                WorkflowStep::new("invariants", "Key invariants (comma-separated)?").skippable(),
                WorkflowStep::new("required", "Required semantics (comma-separated)?").skippable(),
                WorkflowStep::new("forbidden", "Forbidden behavior (comma-separated)?").skippable(),
                WorkflowStep::new("review", "Review the draft contract"),
            ],
            layout,
        }
    }
}

impl Workflow for ContractNewWorkflow {
    type Context = ContractNewContext;
    type Output = Contract;

    fn name(&self) -> &str {
        "contract-new"
    }

    fn steps(&self) -> &[WorkflowStep] {
        &self.steps
    }

    fn initial_context(&self) -> ContractNewContext {
        ContractNewContext {
            title: String::new(),
            description: String::new(),
            scope: String::new(),
            invariants: Vec::new(),
            required: Vec::new(),
            forbidden: Vec::new(),
            draft: None,
        }
    }

    fn execute_step(
        &self,
        step_idx: usize,
        ctx: &mut ContractNewContext,
        input: StepInput,
    ) -> StepOutput {
        match step_idx {
            0 => match input {
                StepInput::UserResponse(s) => { ctx.title = s; StepOutput::Advance }
                _ => StepOutput::Question(Question::simple("Contract title (e.g. Key-Value Store)?")),
            },
            1 => match input {
                StepInput::UserResponse(s) if !s.trim().is_empty() => {
                    ctx.description = s;
                    StepOutput::Advance
                }
                StepInput::UserResponse(_) | StepInput::Skip => {
                    ctx.description = generate_description_with_ai(&self.layout, &ctx.title);
                    StepOutput::Advance
                }
                _ => StepOutput::Question(
                    Question::simple("Describe the contract's purpose (leave empty for AI-assisted)")
                ),
            },
            2 => match input {
                StepInput::UserResponse(s) if !s.trim().is_empty() => {
                    ctx.scope = s;
                    StepOutput::Advance
                }
                StepInput::UserResponse(_) | StepInput::Skip => {
                    ctx.scope = generate_scope_with_ai(&self.layout, &ctx.title);
                    StepOutput::Advance
                }
                _ => StepOutput::Question(
                    Question::simple("What scope does this contract cover? (leave empty for AI-assisted)")
                ),
            },
            3 => match input {
                StepInput::UserResponse(s) => {
                    ctx.invariants = split_csv(&s);
                    StepOutput::Advance
                }
                StepInput::Skip => StepOutput::Advance,
                _ => StepOutput::Question(
                    Question::simple("Key invariants (comma-separated, or leave empty)?")
                ),
            },
            4 => match input {
                StepInput::UserResponse(s) => {
                    ctx.required = split_csv(&s);
                    StepOutput::Advance
                }
                StepInput::Skip => StepOutput::Advance,
                _ => StepOutput::Question(
                    Question::simple("Required semantics (comma-separated, or leave empty)?")
                ),
            },
            5 => match input {
                StepInput::UserResponse(s) => {
                    ctx.forbidden = split_csv(&s);
                    StepOutput::Advance
                }
                StepInput::Skip => StepOutput::Advance,
                _ => StepOutput::Question(
                    Question::simple("Forbidden behavior (comma-separated, or leave empty)?")
                ),
            },
            6 => {
                // Build the draft contract (ID derived from title)
                let draft = build_contract(ctx);
                let preview = toml::to_string_pretty(&draft).unwrap_or_default();
                ctx.draft = Some(draft);

                match input {
                    StepInput::Accept => StepOutput::Advance,
                    _ => StepOutput::Proposal(Proposal {
                        title: format!("Contract: {}", ctx.title),
                        content: preview,
                        format: "toml".to_string(),
                    }),
                }
            }
            _ => StepOutput::Done,
        }
    }

    fn finalize(&self, ctx: ContractNewContext) -> Option<Contract> {
        ctx.draft
    }
}

/// Generate a contract description using AI, falling back to a default if AI is unavailable.
fn generate_description_with_ai(layout: &RepoLayout, title: &str) -> String {
    let fallback = format!("Behavioral contract for {title}.");

    let provider = match crate::core::generate::build_ai_provider(layout, None) {
        Ok(p) => p,
        Err(_) => return fallback,
    };

    let system = "You are a software architect writing contract descriptions for a Rust project. \
        A contract description is a concise paragraph (2-4 sentences) explaining the purpose, context, \
        and importance of the contract. Be specific and technical. Do not include markdown formatting. \
        Respond with only the description text, nothing else.";

    let user_msg = format!(
        "Write a description for a contract titled \"{title}\". \
         Explain what this contract covers, why it exists, and what guarantees it provides."
    );

    match provider.complete(system, &user_msg) {
        Ok(desc) => {
            let trimmed = desc.trim().to_string();
            if trimmed.is_empty() { fallback } else { trimmed }
        }
        Err(_) => fallback,
    }
}

/// Generate a scope description using AI, falling back to a default if AI is unavailable.
fn generate_scope_with_ai(layout: &RepoLayout, title: &str) -> String {
    let fallback = format!("Defines the behavioral contract for {title}");

    let provider = match crate::core::generate::build_ai_provider(layout, None) {
        Ok(p) => p,
        Err(_) => return fallback,
    };

    let system = "You are a software architect writing contract scope descriptions for a Rust project. \
        A contract scope is a concise 1-2 sentence description of what the component does and \
        what behavioral guarantees it provides. Be specific and technical. Do not include markdown formatting. \
        Respond with only the scope text, nothing else.";

    let user_msg = format!(
        "Write a scope description for a contract titled \"{title}\". \
         The scope should describe what this component is responsible for and what guarantees it provides."
    );

    match provider.complete(system, &user_msg) {
        Ok(scope) => {
            let trimmed = scope.trim().to_string();
            if trimmed.is_empty() { fallback } else { trimmed }
        }
        Err(_) => fallback,
    }
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn build_contract(ctx: &ContractNewContext) -> Contract {
    let id = slugify(&ctx.title);
    let mut contract = Contract::new_draft(id, ctx.title.clone(), ctx.scope.clone());
    contract.description = ctx.description.clone();

    for (i, inv) in ctx.invariants.iter().enumerate() {
        contract.invariants.push(Invariant {
            id: format!("inv-{:03}", i + 1),
            description: inv.clone(),
            severity: Severity::Required,
            test_tags: Vec::new(),
        });
    }

    for (i, req) in ctx.required.iter().enumerate() {
        contract.required_semantics.push(Semantic {
            id: format!("req-{:03}", i + 1),
            description: req.clone(),
            test_tags: vec!["conformance".to_string()],
        });
    }

    for (i, forbid) in ctx.forbidden.iter().enumerate() {
        contract.forbidden_semantics.push(Semantic {
            id: format!("forbid-{:03}", i + 1),
            description: forbid.clone(),
            test_tags: vec!["safety".to_string()],
        });
    }

    contract
}

/// Run the `lexicon contract new` flow.
pub fn contract_new(
    layout: &RepoLayout,
    driver: &dyn ConversationDriver,
) -> CoreResult<Option<Contract>> {
    let workflow = ContractNewWorkflow::new(layout.clone());
    let engine = ConversationEngine::new(WorkflowKind::ContractNew);

    let (output, session) = engine.run(&workflow, driver)?;
    save_session(&layout.conversations_dir(), &session)?;

    if let Some(ref contract) = output {
        crate::scaffold::contract::write_contract(layout, contract)?;

        let record = AuditRecord::new(
            AuditAction::ContractCreate,
            Actor::User,
            format!("Created contract: {} ({})", contract.title, contract.id),
        );
        write_audit_record(&layout.audit_dir(), &record)?;
    }

    Ok(output)
}

/// Non-interactive contract creation for testing and CI.
///
/// The contract ID is automatically derived from the title via slugification.
/// If description or scope is empty, AI-assisted generation is attempted (falls back to a default).
pub fn contract_new_noninteractive(
    layout: &RepoLayout,
    title: String,
    description: String,
    scope: String,
    invariants: Vec<String>,
    required: Vec<String>,
    forbidden: Vec<String>,
) -> CoreResult<Contract> {
    let description = if description.trim().is_empty() {
        generate_description_with_ai(layout, &title)
    } else {
        description
    };
    let scope = if scope.trim().is_empty() {
        generate_scope_with_ai(layout, &title)
    } else {
        scope
    };
    let ctx = ContractNewContext {
        title,
        description,
        scope,
        invariants,
        required,
        forbidden,
        draft: None,
    };
    let contract = build_contract(&ctx);

    crate::scaffold::contract::write_contract(layout, &contract)?;

    let record = AuditRecord::new(
        AuditAction::ContractCreate,
        Actor::User,
        format!("Created contract: {} ({})", contract.title, contract.id),
    );
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(contract)
}

/// List all contracts in the repo.
pub fn contract_list(layout: &RepoLayout) -> CoreResult<Vec<String>> {
    Ok(crate::scaffold::contract::list_contracts(layout)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, RepoLayout) {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        crate::core::init::init_repo_noninteractive(
            &layout,
            "test".to_string(),
            "test".to_string(),
            crate::spec::common::RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();
        (dir, layout)
    }

    #[test]
    fn test_contract_new_noninteractive() {
        let (_dir, layout) = setup_repo();

        let contract = contract_new_noninteractive(
            &layout,
            "KV Store".to_string(),
            String::new(),
            "Basic key-value operations".to_string(),
            vec!["get after set returns value".to_string()],
            vec!["get missing returns None".to_string()],
            vec!["must not panic on missing key".to_string()],
        )
        .unwrap();

        // ID is auto-derived from title
        assert_eq!(contract.id, "kv-store");
        assert_eq!(contract.title, "KV Store");
        assert_eq!(contract.invariants.len(), 1);
        assert_eq!(contract.required_semantics.len(), 1);
        assert_eq!(contract.forbidden_semantics.len(), 1);

        // Verify file was written
        let ids = contract_list(&layout).unwrap();
        assert_eq!(ids, vec!["kv-store"]);
    }
}
