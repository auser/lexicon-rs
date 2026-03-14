use lexicon_core::prompt_gen;
use lexicon_repo::layout::RepoLayout;

use crate::app::PromptAction;
use crate::commands::review;
use crate::output;

pub fn run(action: PromptAction) -> miette::Result<()> {
    let layout = RepoLayout::discover()?;

    match action {
        PromptAction::Generate {
            contract_id,
            target,
            ai,
        } => run_generate(&layout, &contract_id, target.as_deref(), ai),
        PromptAction::List => run_list(&layout),
        PromptAction::Status => run_status(&layout),
        PromptAction::Regenerate { prompt, all, ai } => {
            run_regenerate(&layout, prompt.as_deref(), all, ai)
        }
        PromptAction::Explain { prompt } => run_explain(&layout, &prompt),
    }
}

fn run_generate(
    layout: &RepoLayout,
    contract_id: &str,
    target: Option<&str>,
    use_ai: bool,
) -> miette::Result<()> {
    output::heading("Generating implementation prompt");
    output::info(&format!("Contract: {contract_id}"));
    if let Some(t) = target {
        output::info(&format!("Target: {t}"));
    }
    println!();

    let result = prompt_gen::generate_prompt(layout, contract_id, target, use_ai)?;

    review::show_warnings(&result.warnings);
    review::review_artifact(layout, &result.artifact)?;

    Ok(())
}

fn run_list(layout: &RepoLayout) -> miette::Result<()> {
    let prompts = prompt_gen::list_prompts(layout)?;

    if prompts.is_empty() {
        output::info("No prompts found. Run `lexicon prompt generate <contract-id>` to create one.");
        return Ok(());
    }

    output::heading("Implementation Prompts");
    for name in &prompts {
        output::info(name);
    }
    output::info(&format!("\n{} prompt(s) total", prompts.len()));

    Ok(())
}

fn run_status(layout: &RepoLayout) -> miette::Result<()> {
    let statuses = prompt_gen::check_all_prompt_statuses(layout)?;

    if statuses.is_empty() {
        output::info("No prompts found. Run `lexicon prompt generate <contract-id>` to create one.");
        return Ok(());
    }

    output::heading("Prompt Synchronization Status");
    println!();

    let mut stale_count = 0;
    for status in &statuses {
        if status.is_stale {
            stale_count += 1;
            output::warning(&format!("STALE  {}", status.filename));
            for reason in &status.reasons {
                output::info(&format!("       {reason}"));
            }
        } else {
            output::success(&format!("OK     {}", status.filename));
        }
    }

    println!();
    if stale_count > 0 {
        output::warning(&format!(
            "{stale_count} stale prompt(s). Run `lexicon prompt regenerate` to update."
        ));
    } else {
        output::success("All prompts up to date.");
    }

    Ok(())
}

fn run_regenerate(
    layout: &RepoLayout,
    prompt: Option<&str>,
    all: bool,
    use_ai: bool,
) -> miette::Result<()> {
    if let Some(name) = prompt {
        output::heading(&format!("Regenerating prompt: {name}"));
        let result = prompt_gen::regenerate_one(layout, name, use_ai)?;
        review::show_warnings(&result.warnings);
        review::review_artifact(layout, &result.artifact)?;
        return Ok(());
    }

    if all {
        // Regenerate all prompts
        let prompts = prompt_gen::list_prompts(layout)?;
        if prompts.is_empty() {
            output::info("No prompts found.");
            return Ok(());
        }
        output::heading(&format!("Regenerating all {} prompt(s)", prompts.len()));
        for name in &prompts {
            output::info(&format!("Regenerating {name}..."));
            let result = prompt_gen::regenerate_one(layout, name, use_ai)?;
            review::show_warnings(&result.warnings);
            review::review_artifact(layout, &result.artifact)?;
        }
        return Ok(());
    }

    // Default: regenerate only stale
    output::heading("Regenerating stale prompts");
    let results = prompt_gen::regenerate_stale(layout, use_ai)?;

    if results.is_empty() {
        output::success("All prompts are up to date. Nothing to regenerate.");
        return Ok(());
    }

    output::info(&format!("{} stale prompt(s) to regenerate", results.len()));
    println!();

    for result in &results {
        review::show_warnings(&result.warnings);
        review::review_artifact(layout, &result.artifact)?;
    }

    Ok(())
}

fn run_explain(layout: &RepoLayout, prompt_name: &str) -> miette::Result<()> {
    let explanation = prompt_gen::explain_prompt(layout, prompt_name)?;

    output::heading(&format!("Prompt: {}", explanation.node_id));
    output::info(&format!("File: {}", explanation.path));
    if explanation.is_stale {
        output::warning("Status: STALE");
    } else {
        output::success("Status: Up to date");
    }

    println!();
    output::heading("Dependencies");
    for dep in &explanation.dependencies {
        let status = if dep.changed { "CHANGED" } else { "ok" };
        let icon = if dep.changed { "!" } else { " " };
        output::info(&format!(
            " {icon} {:<30} [{status}]",
            dep.source_id
        ));
        output::info(&format!("   Path: {}", dep.source_path));
        if dep.changed {
            output::info(&format!("   Stored hash:  {}...", &dep.stored_hash[..12.min(dep.stored_hash.len())]));
            output::info(&format!("   Current hash: {}...", &dep.current_hash[..12.min(dep.current_hash.len())]));
        }
    }

    if explanation.is_stale {
        println!();
        output::info("Run `lexicon prompt regenerate` to update this prompt.");
    }

    Ok(())
}
