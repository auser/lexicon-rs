use lexicon_rs::core::verify::verify;
use lexicon_rs::gates::result::GateOutcome;
use lexicon_rs::repo::layout::RepoLayout;
use lexicon_rs::scoring::engine::Verdict;

use crate::output;

pub fn run(health: bool) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("Running verification");
    output::divider();

    let result = verify(&layout)?;

    // Display gate results
    output::heading("Gates");
    for gr in &result.gate_results {
        let icon = match gr.outcome {
            GateOutcome::Pass => "✓",
            GateOutcome::Fail => "✗",
            GateOutcome::Skip => "⊘",
            GateOutcome::Error => "!",
        };
        let line = format!("{icon} {} ({}ms)", gr.gate_id, gr.duration_ms);
        match gr.outcome {
            GateOutcome::Pass => output::success(&line),
            GateOutcome::Fail => output::error(&line),
            GateOutcome::Skip => output::warning(&line),
            GateOutcome::Error => output::error(&line),
        }
    }

    // Display score
    if let Some(ref report) = result.score_report {
        println!();
        output::heading("Score");
        let verdict_str = match report.verdict {
            Verdict::Pass => "PASS",
            Verdict::Warn => "WARN",
            Verdict::Fail => "FAIL",
        };
        output::info(&format!(
            "Total: {:.1}% ({})",
            report.total_score * 100.0,
            verdict_str
        ));
        for dim in &report.dimensions {
            output::info(&format!(
                "  {}: {:.0}% — {}",
                dim.dimension_id,
                dim.value * 100.0,
                dim.explanation
            ));
        }
    }

    // Display coverage report
    if let Some(ref cov) = result.coverage_report {
        println!();
        output::heading("Contract Coverage");
        output::info(&format!(
            "Overall: {:.1}% ({}/{} clauses covered)",
            cov.overall_coverage_pct, cov.total_covered, cov.total_clauses
        ));
        if !cov.uncovered_clauses.is_empty() {
            output::warning(&format!(
                "{} uncovered clause(s)",
                cov.uncovered_clauses.len()
            ));
        }
    }

    // Display API drift
    if let Some(ref diff) = result.api_diff {
        println!();
        output::heading("API Drift");
        if diff.is_empty() {
            output::success("No API changes from baseline");
        } else {
            output::info(&diff.summary());
            if diff.has_breaking() {
                output::error(&format!("{} breaking change(s)", diff.breaking_count()));
            }
        }
    }

    // Display prompt staleness warnings
    if !result.prompt_warnings.is_empty() {
        println!();
        output::heading("Prompt Synchronization");
        for w in &result.prompt_warnings {
            output::warning(w);
        }
        output::info("Use REGENERATE_PROMPTS in `lexicon chat` to update stale prompts.");
    }

    output::divider();

    let all_passed = result
        .gate_results
        .iter()
        .all(|r| r.outcome == GateOutcome::Pass || r.outcome == GateOutcome::Skip);

    if all_passed {
        output::success("All gates passed");
    } else {
        output::error("Some gates failed");
    }

    // Health checks (manifest, gates, scoring, CLAUDE.md, API baseline)
    if health {
        println!();
        output::heading("Health Checks");

        if layout.manifest_path().exists() {
            output::success("✓ Manifest found");
        } else {
            output::error("✗ No manifest — run `lexicon init`");
        }

        if layout.gates_path().exists() {
            output::success("✓ Gates configured");
        } else {
            output::warning("⚠ No gates configured");
        }

        if layout.scoring_model_path().exists() {
            output::success("✓ Scoring model configured");
        } else {
            output::warning("⚠ No scoring model");
        }

        if layout.claude_md_path().exists() {
            output::success("✓ CLAUDE.md present");
        } else {
            output::warning("⚠ No CLAUDE.md");
        }

        if layout.api_dir().join("baseline.json").exists() {
            output::success("✓ API baseline present");
        } else {
            output::warning("⚠ No API baseline — use DOCTOR in chat to create one");
        }
    }

    Ok(())
}
