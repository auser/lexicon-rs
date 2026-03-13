use lexicon_core::api;
use lexicon_repo::layout::RepoLayout;

use crate::app::ApiAction;
use crate::output;

pub fn run(action: ApiAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        ApiAction::Scan => {
            output::heading("Scanning public API");
            let snapshot = api::api_scan(&layout)?;
            output::success(&format!("{} API items extracted", snapshot.items.len()));
            output::info("Saved to .lexicon/api/current.json");
        }
        ApiAction::Diff => {
            if !api::has_baseline(&layout) {
                output::error("No baseline found — run `lexicon api baseline` first");
                return Ok(());
            }
            if !api::has_current_scan(&layout) {
                output::info("Running API scan first...");
                api::api_scan(&layout)?;
            }
            let diff = api::api_diff(&layout)?;
            if diff.is_empty() {
                output::success("No API changes from baseline");
            } else {
                if diff.has_breaking() {
                    output::error(&format!(
                        "{} breaking change(s) detected",
                        diff.breaking_count()
                    ));
                }
                output::info(&diff.summary());
            }
        }
        ApiAction::Report { json } => {
            if !api::has_current_scan(&layout) {
                api::api_scan(&layout)?;
            }
            if !api::has_baseline(&layout) {
                output::warning(
                    "No baseline — showing current API only. Run `lexicon api baseline` to set a baseline.",
                );
                return Ok(());
            }
            if json {
                let report = api::api_report_json(&layout)?;
                println!("{report}");
            } else {
                let report = api::api_report(&layout)?;
                println!("{report}");
            }
        }
        ApiAction::Baseline => {
            if !api::has_current_scan(&layout) {
                output::info("Running API scan first...");
                api::api_scan(&layout)?;
            }
            api::api_baseline(&layout)?;
            output::success("Current API saved as baseline");
        }
    }
    Ok(())
}
