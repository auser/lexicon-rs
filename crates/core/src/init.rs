use lexicon_audit::writer::write_audit_record;
use lexicon_conversation::driver::ConversationDriver;
use lexicon_conversation::engine::ConversationEngine;
use lexicon_conversation::session::save_session;
use lexicon_conversation::workflow::{
    Question, StepInput, StepOutput, Workflow, WorkflowStep,
};
use lexicon_repo::detect::{detect_shape, RepoShape};
use lexicon_repo::inspect::inspect_repo;
use lexicon_repo::layout::RepoLayout;
use lexicon_spec::audit::AuditRecord;
use lexicon_spec::common::{Actor, AuditAction, RepoType, WorkflowKind};
use lexicon_spec::manifest::Manifest;

use crate::error::CoreResult;

/// The init workflow context.
struct InitContext {
    name: String,
    description: String,
    repo_type: RepoType,
    domain: String,
}

/// Workflow for `lexicon init`.
struct InitWorkflow {
    detected_name: Option<String>,
    detected_type: RepoType,
    steps: Vec<WorkflowStep>,
}

impl InitWorkflow {
    fn new(layout: &RepoLayout) -> Self {
        let (detected_name, detected_type) = inspect_repo(&layout.root)
            .map(|info| (info.name, info.repo_type))
            .unwrap_or((None, RepoType::Library));

        Self {
            detected_name,
            detected_type,
            steps: vec![
                WorkflowStep::new("name", "What is the project name?"),
                WorkflowStep::new("description", "Brief project description?"),
                WorkflowStep::new("domain", "What domain is this project in? (e.g., parser, key-value store, web framework)"),
            ],
        }
    }
}

impl Workflow for InitWorkflow {
    type Context = InitContext;
    type Output = Manifest;

    fn name(&self) -> &str {
        "init"
    }

    fn steps(&self) -> &[WorkflowStep] {
        &self.steps
    }

    fn initial_context(&self) -> InitContext {
        InitContext {
            name: self.detected_name.clone().unwrap_or_default(),
            description: String::new(),
            repo_type: self.detected_type,
            domain: String::new(),
        }
    }

    fn execute_step(&self, step_idx: usize, ctx: &mut InitContext, input: StepInput) -> StepOutput {
        match step_idx {
            0 => match input {
                StepInput::UserResponse(s) => {
                    ctx.name = s;
                    StepOutput::Advance
                }
                _ => {
                    let mut q = Question::simple("What is the project name?");
                    if let Some(ref name) = self.detected_name {
                        q = q.with_default(name.clone());
                    }
                    StepOutput::Question(q)
                }
            },
            1 => match input {
                StepInput::UserResponse(s) => {
                    ctx.description = s;
                    StepOutput::Advance
                }
                _ => StepOutput::Question(Question::simple("Brief project description?")),
            },
            2 => match input {
                StepInput::UserResponse(s) => {
                    ctx.domain = s;
                    StepOutput::Advance
                }
                _ => StepOutput::Question(
                    Question::simple("What domain is this project in? (e.g., parser, key-value store)")
                ),
            },
            _ => StepOutput::Done,
        }
    }

    fn finalize(&self, ctx: InitContext) -> Option<Manifest> {
        if ctx.name.is_empty() {
            return None;
        }
        Some(Manifest::new(ctx.name, ctx.description, ctx.repo_type, ctx.domain))
    }
}

/// Run the `lexicon init` flow.
pub fn init_repo(
    layout: &RepoLayout,
    driver: &dyn ConversationDriver,
) -> CoreResult<()> {
    let workflow = InitWorkflow::new(layout);
    let engine = ConversationEngine::new(WorkflowKind::Init);

    let (output, session) = engine.run(&workflow, driver)?;

    // Save session
    save_session(&layout.conversations_dir(), &session)?;

    if let Some(manifest) = output {
        // Initialize the repo
        lexicon_scaffold::init::init_repo(layout, &manifest)?;

        // Initialize default gates and scoring model
        crate::score::gate_init(layout)?;
        eprintln!("  Default gates initialized");
        crate::score::score_init(layout)?;
        eprintln!("  Default scoring model initialized");

        // Detect repo shape and print workspace hint
        print_workspace_hint(layout);

        // Write audit record
        let record = AuditRecord::new(
            AuditAction::RepoInit,
            Actor::User,
            format!("Initialized lexicon for project: {}", manifest.project.name),
        );
        write_audit_record(&layout.audit_dir(), &record)?;
    }

    Ok(())
}

/// Non-interactive init for testing and CI.
pub fn init_repo_noninteractive(
    layout: &RepoLayout,
    name: String,
    description: String,
    repo_type: RepoType,
    domain: String,
) -> CoreResult<()> {
    let manifest = Manifest::new(name.clone(), description, repo_type, domain);
    lexicon_scaffold::init::init_repo(layout, &manifest)?;

    // Initialize default gates and scoring model
    crate::score::gate_init(layout)?;
    crate::score::score_init(layout)?;

    // Detect repo shape and print workspace hint
    print_workspace_hint(layout);

    let record = AuditRecord::new(
        AuditAction::RepoInit,
        Actor::User,
        format!("Initialized lexicon for project: {name}"),
    );
    write_audit_record(&layout.audit_dir(), &record)?;

    Ok(())
}

/// Detect the repo shape and print an informational message if it's a workspace.
fn print_workspace_hint(layout: &RepoLayout) {
    let shape = detect_shape(&layout.root);
    if let RepoShape::Workspace { member_count } = shape {
        eprintln!(
            "Detected Cargo workspace with {} members. Using Repo Mode. \
             Run `lexicon workspace init` to enable workspace governance.",
            member_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_noninteractive() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());

        init_repo_noninteractive(
            &layout,
            "test-project".to_string(),
            "A test".to_string(),
            RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();

        assert!(layout.manifest_path().exists());
        assert!(layout.contracts_dir().is_dir());
        assert!(layout.audit_dir().is_dir());
        assert!(layout.gates_path().exists());
        assert!(layout.scoring_model_path().exists());

        // Verify audit record was written
        let records = lexicon_audit::reader::list_audit_records(&layout.audit_dir()).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].action, AuditAction::RepoInit);
    }
}
