use lexicon_repo::layout::RepoLayout;

use crate::output;

pub fn run() -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    output::heading("Doctor — checking repo health");

    // Check manifest
    if layout.manifest_path().exists() {
        output::success("Manifest found");
    } else {
        output::error("No manifest — run `lexicon init` first");
        return Ok(());
    }

    // Check contracts dir
    let contracts = lexicon_core::contract::contract_list(&layout)?;
    output::info(&format!("{} contract(s)", contracts.len()));

    // Check scoring model
    if layout.scoring_model_path().exists() {
        output::success("Scoring model configured");
    } else {
        output::warning("No scoring model — run `lexicon score init`");
    }

    // Check gates
    if layout.gates_path().exists() {
        output::success("Gates configured");
    } else {
        output::warning("No gates — run `lexicon gate init`");
    }

    // Check CLAUDE.md
    if layout.claude_md_path().exists() {
        output::success("CLAUDE.md present");
    } else {
        output::warning("No CLAUDE.md — run `lexicon sync claude`");
    }

    // Check API baseline
    let api_dir = layout.api_dir();
    if api_dir.join("baseline.json").exists() {
        output::success("API baseline configured");
        if api_dir.join("current.json").exists() {
            // Try to detect drift
            match lexicon_core::api::api_diff(&layout) {
                Ok(diff) => {
                    if diff.is_empty() {
                        output::success("No API drift from baseline");
                    } else {
                        if diff.has_breaking() {
                            output::error(&format!(
                                "API drift: {} breaking change(s)",
                                diff.breaking_count()
                            ));
                        } else {
                            output::warning(&format!("API drift: {}", diff.summary()));
                        }
                    }
                }
                Err(_) => output::warning("Could not compute API diff"),
            }
        } else {
            output::warning("No current API scan — run `lexicon api scan`");
        }
    } else {
        output::info("No API baseline — run `lexicon api scan && lexicon api baseline`");
    }

    // Check contract coverage
    let contracts = lexicon_core::contract::contract_list(&layout)?;
    if !contracts.is_empty() {
        // Load contracts and check coverage
        let contract_files: Vec<_> = contracts
            .iter()
            .filter_map(|id| {
                let path = layout.contracts_dir().join(format!("{id}.toml"));
                std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|c| toml::from_str::<lexicon_spec::contract::Contract>(&c).ok())
            })
            .collect();

        if let Ok(report) = lexicon_core::coverage::coverage_report(&layout, &contract_files) {
            if report.total_clauses > 0 {
                let msg = format!(
                    "Contract coverage: {:.1}% ({}/{} clauses)",
                    report.overall_coverage_pct, report.total_covered, report.total_clauses
                );
                if report.overall_coverage_pct >= 80.0 {
                    output::success(&msg);
                } else if report.overall_coverage_pct >= 50.0 {
                    output::warning(&msg);
                } else {
                    output::error(&msg);
                }
                if !report.uncovered_clauses.is_empty() {
                    for uc in &report.uncovered_clauses {
                        output::warning(&format!(
                            "  Uncovered: [{}] {} — {}",
                            uc.contract_id, uc.clause_id, uc.description
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
