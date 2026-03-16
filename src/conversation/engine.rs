use crate::spec::common::{StepType, WorkflowKind};
use crate::spec::session::ConversationSession;

use crate::conversation::driver::{ConversationDriver, ProposalResponse};
use crate::conversation::error::{ConversationError, ConversationResult};
use crate::conversation::workflow::{StepInput, StepOutput, Workflow};

/// The conversation engine drives a workflow through its steps.
pub struct ConversationEngine {
    workflow_kind: WorkflowKind,
}

impl ConversationEngine {
    pub fn new(workflow_kind: WorkflowKind) -> Self {
        Self { workflow_kind }
    }

    /// Run a workflow to completion, returning the output and session record.
    pub fn run<W: Workflow>(
        &self,
        workflow: &W,
        driver: &dyn ConversationDriver,
    ) -> ConversationResult<(Option<W::Output>, ConversationSession)> {
        let mut session = ConversationSession::new(self.workflow_kind);
        let mut ctx = workflow.initial_context();
        let steps = workflow.steps();

        session.add_step(
            StepType::Info,
            format!("Starting workflow: {}", workflow.name()),
        );

        for (idx, step) in steps.iter().enumerate() {
            // Execute the step to get what we should show the user
            let output = workflow.execute_step(idx, &mut ctx, StepInput::Skip);

            match output {
                StepOutput::Question(question) => {
                    session.add_step(StepType::Question, question.text.clone());
                    let response = driver.present_question(&question)?;
                    session.add_step(StepType::UserInput, response.clone());

                    // Re-execute with the actual user input
                    workflow.execute_step(
                        idx,
                        &mut ctx,
                        StepInput::UserResponse(response.clone()),
                    );

                    session.add_decision(step.id.clone(), response, None);
                }
                StepOutput::Proposal(proposal) => {
                    session.add_step(StepType::Proposal, proposal.content.clone());

                    match driver.present_proposal(&proposal)? {
                        ProposalResponse::Accept => {
                            workflow.execute_step(idx, &mut ctx, StepInput::Accept);
                            session.add_step(StepType::UserInput, "accepted".to_string());
                        }
                        ProposalResponse::Refine(feedback) => {
                            session.add_step(StepType::Refinement, feedback.clone());
                            workflow.execute_step(
                                idx,
                                &mut ctx,
                                StepInput::Refine(feedback),
                            );
                        }
                        ProposalResponse::Skip => {
                            if step.skippable {
                                workflow.execute_step(idx, &mut ctx, StepInput::Skip);
                                session.add_step(StepType::UserInput, "skipped".to_string());
                            }
                        }
                        ProposalResponse::Abort => {
                            session.abandon();
                            return Err(ConversationError::Aborted);
                        }
                    }
                }
                StepOutput::Info(message) => {
                    driver.present_info(&message);
                    session.add_step(StepType::Info, message);
                }
                StepOutput::Advance => {
                    // Step completed silently
                }
                StepOutput::Done => {
                    break;
                }
            }
        }

        let output = workflow.finalize(ctx);
        let artifact_id = if output.is_some() {
            Some(workflow.name().to_string())
        } else {
            None
        };
        session.complete(artifact_id);

        Ok((output, session))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::driver::MockResponse;
    use crate::conversation::workflow::{Question, WorkflowStep};
    use crate::spec::common::SessionStatus;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct TestWorkflow;

    impl Workflow for TestWorkflow {
        type Context = Vec<String>;
        type Output = String;

        fn name(&self) -> &str {
            "test-workflow"
        }

        fn steps(&self) -> &[WorkflowStep] {
            &[]
        }

        fn initial_context(&self) -> Vec<String> {
            Vec::new()
        }

        fn execute_step(
            &self,
            step_idx: usize,
            ctx: &mut Vec<String>,
            input: StepInput,
        ) -> StepOutput {
            match step_idx {
                0 => match input {
                    StepInput::UserResponse(s) => {
                        ctx.push(s);
                        StepOutput::Advance
                    }
                    _ => StepOutput::Question(Question::simple("What is your name?")),
                },
                _ => StepOutput::Done,
            }
        }

        fn finalize(&self, ctx: Vec<String>) -> Option<String> {
            Some(ctx.join(", "))
        }
    }

    // Minimal test of the workflow trait itself
    #[test]
    fn test_workflow_trait() {
        let wf = TestWorkflow;
        assert_eq!(wf.name(), "test-workflow");
        let mut ctx = wf.initial_context();
        assert!(ctx.is_empty());

        let output = wf.execute_step(0, &mut ctx, StepInput::UserResponse("Alice".to_string()));
        assert!(matches!(output, StepOutput::Advance));
        assert_eq!(ctx, vec!["Alice".to_string()]);

        let result = wf.finalize(ctx);
        assert_eq!(result, Some("Alice".to_string()));
    }

    #[test]
    fn test_engine_with_mock_driver() {
        // This test verifies the engine can drive a minimal workflow
        // Since TestWorkflow has no steps() defined, it should complete immediately
        let wf = TestWorkflow;
        let engine = ConversationEngine::new(WorkflowKind::Init);
        let responses: VecDeque<MockResponse> = VecDeque::new();
        let driver = RefCell::new(responses);

        let (output, session) = engine.run(&wf, &driver).unwrap();
        assert!(output.is_some());
        assert_eq!(session.status, SessionStatus::Completed);
    }
}
