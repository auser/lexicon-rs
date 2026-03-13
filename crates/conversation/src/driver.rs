use crate::error::{ConversationError, ConversationResult};
use crate::workflow::{Proposal, Question};

/// How the user responded to a proposal.
#[derive(Debug, Clone)]
pub enum ProposalResponse {
    /// Accept the proposal as-is.
    Accept,
    /// Refine with feedback.
    Refine(String),
    /// Skip this step.
    Skip,
    /// Abort the workflow.
    Abort,
}

/// Trait for driving conversation I/O.
///
/// The driver handles all user interaction. This abstraction allows
/// the same workflow to be driven by CLI prompts, TUI panels, or
/// automated test harnesses.
pub trait ConversationDriver {
    /// Present a question and collect the user's response.
    fn present_question(&self, question: &Question) -> ConversationResult<String>;

    /// Present a proposal and collect the user's response.
    fn present_proposal(&self, proposal: &Proposal) -> ConversationResult<ProposalResponse>;

    /// Display an informational message.
    fn present_info(&self, message: &str);
}

/// A terminal-based conversation driver using dialoguer.
pub struct TerminalDriver;

impl ConversationDriver for TerminalDriver {
    fn present_question(&self, question: &Question) -> ConversationResult<String> {
        use dialoguer::Input;

        let mut input = Input::<String>::new().with_prompt(&question.text);

        if let Some(ref default) = question.default {
            input = input.default(default.clone());
        }

        input
            .interact_text()
            .map_err(|e| ConversationError::Dialoguer(e.to_string()))
    }

    fn present_proposal(&self, proposal: &Proposal) -> ConversationResult<ProposalResponse> {
        use console::style;
        use dialoguer::Select;

        println!("\n{}", style(&proposal.title).bold().cyan());
        println!("{}", style("─".repeat(60)).dim());
        println!("{}", &proposal.content);
        println!("{}", style("─".repeat(60)).dim());

        let choices = &["Accept", "Refine", "Skip", "Abort"];
        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .items(choices)
            .default(0)
            .interact()
            .map_err(|e| ConversationError::Dialoguer(e.to_string()))?;

        match selection {
            0 => Ok(ProposalResponse::Accept),
            1 => {
                let feedback: String = dialoguer::Input::new()
                    .with_prompt("Feedback")
                    .interact_text()
                    .map_err(|e| ConversationError::Dialoguer(e.to_string()))?;
                Ok(ProposalResponse::Refine(feedback))
            }
            2 => Ok(ProposalResponse::Skip),
            3 => Ok(ProposalResponse::Abort),
            _ => Ok(ProposalResponse::Accept),
        }
    }

    fn present_info(&self, message: &str) {
        use console::style;
        println!("{} {}", style("ℹ").blue(), message);
    }
}

/// A test driver that plays back pre-defined responses.
#[cfg(test)]
pub struct MockDriver {
    pub responses: Vec<MockResponse>,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub enum MockResponse {
    Text(String),
    AcceptProposal,
    SkipProposal,
}

#[cfg(test)]
impl MockDriver {
    pub fn new(responses: Vec<MockResponse>) -> Self {
        Self { responses }
    }
}

#[cfg(test)]
impl ConversationDriver for std::cell::RefCell<std::collections::VecDeque<MockResponse>> {
    fn present_question(&self, _question: &Question) -> ConversationResult<String> {
        match self.borrow_mut().pop_front() {
            Some(MockResponse::Text(s)) => Ok(s),
            _ => Ok(String::new()),
        }
    }

    fn present_proposal(&self, _proposal: &Proposal) -> ConversationResult<ProposalResponse> {
        match self.borrow_mut().pop_front() {
            Some(MockResponse::AcceptProposal) => Ok(ProposalResponse::Accept),
            Some(MockResponse::SkipProposal) => Ok(ProposalResponse::Skip),
            _ => Ok(ProposalResponse::Accept),
        }
    }

    fn present_info(&self, _message: &str) {
        // no-op in tests
    }
}
