use std::path::Path;
use std::process::Command;
use std::time::Instant;

use lexicon_spec::gates::Gate;

use crate::error::GatesResult;
use crate::result::{GateOutcome, GateResult};

/// Run a single gate command in the given working directory.
pub fn run_gate(gate: &Gate, working_dir: &Path) -> GatesResult<GateResult> {
    let start = Instant::now();

    let output = Command::new("sh")
        .arg("-c")
        .arg(&gate.command)
        .current_dir(working_dir)
        .output();

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(output) => {
            let outcome = if output.status.success() {
                GateOutcome::Pass
            } else {
                GateOutcome::Fail
            };

            Ok(GateResult {
                gate_id: gate.id.clone(),
                outcome,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms,
            })
        }
        Err(e) => Ok(GateResult {
            gate_id: gate.id.clone(),
            outcome: GateOutcome::Error,
            stdout: String::new(),
            stderr: e.to_string(),
            duration_ms,
        }),
    }
}

/// Run all gates in a gates model, returning results for each.
pub fn run_all_gates(
    gates: &[Gate],
    working_dir: &Path,
    skip_ids: &[String],
) -> GatesResult<Vec<GateResult>> {
    let mut results = Vec::new();

    for gate in gates {
        if skip_ids.contains(&gate.id) {
            results.push(GateResult {
                gate_id: gate.id.clone(),
                outcome: GateOutcome::Skip,
                stdout: String::new(),
                stderr: String::new(),
                duration_ms: 0,
            });
            continue;
        }

        results.push(run_gate(gate, working_dir)?);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_spec::common::DimensionCategory;
    use tempfile::TempDir;

    fn test_gate(id: &str, command: &str) -> Gate {
        Gate {
            id: id.to_string(),
            label: id.to_string(),
            command: command.to_string(),
            category: DimensionCategory::Scored,
            timeout_secs: Some(10),
            allow_skip: true,
        }
    }

    #[test]
    fn test_run_passing_gate() {
        let dir = TempDir::new().unwrap();
        let gate = test_gate("echo-test", "echo hello");
        let result = run_gate(&gate, dir.path()).unwrap();
        assert_eq!(result.outcome, GateOutcome::Pass);
        assert!(result.stdout.contains("hello"));
    }

    #[test]
    fn test_run_failing_gate() {
        let dir = TempDir::new().unwrap();
        let gate = test_gate("fail-test", "exit 1");
        let result = run_gate(&gate, dir.path()).unwrap();
        assert_eq!(result.outcome, GateOutcome::Fail);
    }

    #[test]
    fn test_run_all_with_skip() {
        let dir = TempDir::new().unwrap();
        let gates = vec![
            test_gate("a", "echo a"),
            test_gate("b", "echo b"),
        ];
        let results = run_all_gates(&gates, dir.path(), &["b".to_string()]).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].outcome, GateOutcome::Pass);
        assert_eq!(results[1].outcome, GateOutcome::Skip);
    }
}
