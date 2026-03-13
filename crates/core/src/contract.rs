use lexicon_audit::writer::write_audit_record;
use lexicon_conversation::driver::ConversationDriver;
use lexicon_conversation::engine::ConversationEngine;
use lexicon_conversation::session::save_session;
use lexicon_conversation::workflow::{
    Proposal, Question, StepInput, StepOutput, Workflow, WorkflowStep,
};
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::common::{Actor, AuditAction, Severity, WorkflowKind};
use lexicon_spec::contract::{Contract, Invariant, Semantic};

use crate::error::CoreResult;

struct ContractNewContext {
    id: String,
    title: String,
    scope: String,
    invariants: Vec<String>,
    required: Vec<String>,
    forbidden: Vec<String>,
    draft: Option<Contract>,
}

struct ContractNewWorkflow {
    steps: Vec<WorkflowStep>,
}

impl ContractNewWorkflow {
    fn new() -> Self {
        Self {
            steps: vec![
                WorkflowStep::new("id", "Contract ID (kebab-case slug, e.g. key-value-store)?"),
                WorkflowStep::new("title", "Contract title?"),
                WorkflowStep::new("scope", "What scope does this contract cover?"),
                WorkflowStep::new("invariants", "Key invariants (comma-separated)?").skippable(),
                WorkflowStep::new("required", "Required semantics (comma-separated)?").skippable(),
                WorkflowStep::new("forbidden", "Forbidden behavior (comma-separated)?").skippable(),
                WorkflowStep::new("review", "Review the draft contract"),
            ],
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
            id: String::new(),
            title: String::new(),
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
                StepInput::UserResponse(s) => { ctx.id = s; StepOutput::Advance }
                _ => StepOutput::Question(Question::simple("Contract ID (kebab-case slug)?")),
            },
            1 => match input {
                StepInput::UserResponse(s) => { ctx.title = s; StepOutput::Advance }
                _ => StepOutput::Question(Question::simple("Contract title?")),
            },
            2 => match input {
                StepInput::UserResponse(s) => { ctx.scope = s; StepOutput::Advance }
                _ => StepOutput::Question(Question::simple("What scope does this contract cover?")),
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
                // Build the draft contract
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

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

fn build_contract(ctx: &ContractNewContext) -> Contract {
    let mut contract = Contract::new_draft(ctx.id.clone(), ctx.title.clone(), ctx.scope.clone());

    for (i, inv) in ctx.invariants.iter().enumerate() {
        contract.invariants.push(Invariant {
            id: format!("inv-{:03}", i + 1),
            description: inv.clone(),
            severity: Severity::Required,
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
    let workflow = ContractNewWorkflow::new();
    let engine = ConversationEngine::new(WorkflowKind::ContractNew);

    let (output, session) = engine.run(&workflow, driver)?;
    save_session(&layout.conversations_dir(), &session)?;

    if let Some(ref contract) = output {
        lexicon_scaffold::contract::write_contract(layout, contract)?;

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
pub fn contract_new_noninteractive(
    layout: &RepoLayout,
    id: String,
    title: String,
    scope: String,
    invariants: Vec<String>,
    required: Vec<String>,
    forbidden: Vec<String>,
) -> CoreResult<Contract> {
    let ctx = ContractNewContext {
        id,
        title,
        scope,
        invariants,
        required,
        forbidden,
        draft: None,
    };
    let contract = build_contract(&ctx);

    lexicon_scaffold::contract::write_contract(layout, &contract)?;

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
    Ok(lexicon_scaffold::contract::list_contracts(layout)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> (TempDir, RepoLayout) {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        crate::init::init_repo_noninteractive(
            &layout,
            "test".to_string(),
            "test".to_string(),
            lexicon_spec::common::RepoType::Library,
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
            "kv-store".to_string(),
            "KV Store".to_string(),
            "Basic key-value operations".to_string(),
            vec!["get after set returns value".to_string()],
            vec!["get missing returns None".to_string()],
            vec!["must not panic on missing key".to_string()],
        )
        .unwrap();

        assert_eq!(contract.id, "kv-store");
        assert_eq!(contract.invariants.len(), 1);
        assert_eq!(contract.required_semantics.len(), 1);
        assert_eq!(contract.forbidden_semantics.len(), 1);

        // Verify file was written
        let ids = contract_list(&layout).unwrap();
        assert_eq!(ids, vec!["kv-store"]);
    }
}
