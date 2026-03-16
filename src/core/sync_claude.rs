use crate::ai::context::assemble_context;
use crate::repo::layout::RepoLayout;
use crate::repo::lexicon_dir::load_manifest;
use crate::scaffold::contract::load_contract;
use crate::spec::contract::Contract;

use crate::core::error::CoreResult;

/// Sync CLAUDE.md with current repo state.
pub fn sync_claude(layout: &RepoLayout) -> CoreResult<()> {
    let manifest = load_manifest(layout)?;

    // Load all contracts
    let contract_ids = crate::scaffold::contract::list_contracts(layout)?;
    let mut contracts: Vec<Contract> = Vec::new();
    for id in &contract_ids {
        if let Some(contract) = load_contract(layout, id)? {
            contracts.push(contract);
        }
    }

    // Load models
    let score_model = crate::scaffold::scoring::load_score_model(layout)?;
    let gates_model = crate::scaffold::gates::load_gates_model(layout)?;

    // Assemble context
    let context = assemble_context(
        &manifest,
        &contracts,
        score_model.as_ref(),
        gates_model.as_ref(),
    );

    // Write to CLAUDE.md
    crate::scaffold::claude::sync_claude_md(layout, &context)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sync_claude_basic() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        crate::core::init::init_repo_noninteractive(
            &layout,
            "my-lib".to_string(),
            "A library".to_string(),
            crate::spec::common::RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();

        sync_claude(&layout).unwrap();

        let content = std::fs::read_to_string(layout.claude_md_path()).unwrap();
        assert!(content.contains("my-lib"));
        assert!(content.contains("lexicon:begin:lexicon-context"));
    }
}
