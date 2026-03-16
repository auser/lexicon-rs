/// A conversational workflow that guides artifact creation.
///
/// Workflows are state machines: they define a sequence of steps,
/// each of which collects input or presents output. The workflow
/// works fully without AI — AI is an optional enhancer.
pub trait Workflow {
    /// The accumulated context built during the workflow.
    type Context;
    /// The final output artifact.
    type Output;

    /// The name of this workflow (for display and session recording).
    fn name(&self) -> &str;

    /// The list of step definitions.
    fn steps(&self) -> &[WorkflowStep];

    /// Build the initial context by inspecting the environment.
    fn initial_context(&self) -> Self::Context;

    /// Execute a single step, updating context based on user input.
    fn execute_step(
        &self,
        step_idx: usize,
        ctx: &mut Self::Context,
        input: StepInput,
    ) -> StepOutput;

    /// Finalize the workflow, producing the output artifact.
    fn finalize(&self, ctx: Self::Context) -> Option<Self::Output>;
}

/// Definition of a workflow step.
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    /// Unique step identifier.
    pub id: String,
    /// Human-readable prompt/description.
    pub prompt: String,
    /// Whether this step can be skipped.
    pub skippable: bool,
}

impl WorkflowStep {
    pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            skippable: false,
        }
    }

    pub fn skippable(mut self) -> Self {
        self.skippable = true;
        self
    }
}

/// Input from the user at a workflow step.
#[derive(Debug, Clone)]
pub enum StepInput {
    /// User typed a text response.
    UserResponse(String),
    /// User accepted a proposal.
    Accept,
    /// User requested refinement with feedback.
    Refine(String),
    /// User skipped this step.
    Skip,
}

/// Output from a workflow step.
#[derive(Debug, Clone)]
pub enum StepOutput {
    /// Ask the user a question.
    Question(Question),
    /// Present a draft artifact for review.
    Proposal(Proposal),
    /// Show informational text.
    Info(String),
    /// Step completed, advance to next.
    Advance,
    /// Workflow is complete.
    Done,
}

/// A question to ask the user.
#[derive(Debug, Clone)]
pub struct Question {
    /// The question text.
    pub text: String,
    /// Optional default answer.
    pub default: Option<String>,
    /// Optional suggestions.
    pub suggestions: Vec<String>,
    /// Whether to accept multiple lines.
    pub multiline: bool,
}

impl Question {
    pub fn simple(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            default: None,
            suggestions: Vec::new(),
            multiline: false,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }
}

/// A proposal for the user to review.
#[derive(Debug, Clone)]
pub struct Proposal {
    /// Title of what's being proposed.
    pub title: String,
    /// The proposed content (rendered as preview).
    pub content: String,
    /// Format hint for rendering (e.g., "toml", "rust", "markdown").
    pub format: String,
}
