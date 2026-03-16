/// The result of running a single gate.
#[derive(Debug, Clone)]
pub struct GateResult {
    /// Gate ID that was run.
    pub gate_id: String,
    /// Outcome of the gate.
    pub outcome: GateOutcome,
    /// Standard output from the gate command.
    pub stdout: String,
    /// Standard error from the gate command.
    pub stderr: String,
    /// How long the gate took to run, in milliseconds.
    pub duration_ms: u64,
}

/// Possible outcomes for a gate execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateOutcome {
    /// Gate passed (exit code 0).
    Pass,
    /// Gate failed (non-zero exit code).
    Fail,
    /// Gate was skipped (allowed by policy).
    Skip,
    /// Gate errored (could not be executed).
    Error,
}

impl GateResult {
    pub fn passed(&self) -> bool {
        self.outcome == GateOutcome::Pass
    }

    pub fn failed(&self) -> bool {
        self.outcome == GateOutcome::Fail
    }
}
