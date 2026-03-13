use lexicon_repo::layout::RepoLayout;
use lexicon_spec::contract::Contract;

use crate::error::ScaffoldResult;

/// Write a contract to the specs/contracts/ directory.
pub fn write_contract(layout: &RepoLayout, contract: &Contract) -> ScaffoldResult<()> {
    let path = layout.contracts_dir().join(format!("{}.toml", contract.id));
    let content = toml::to_string_pretty(contract)?;
    lexicon_fs::safe_write::safe_write(&path, &content, false)?;
    Ok(())
}

/// Load a contract from the specs/contracts/ directory.
pub fn load_contract(layout: &RepoLayout, id: &str) -> ScaffoldResult<Option<Contract>> {
    let path = layout.contracts_dir().join(format!("{id}.toml"));
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let contract: Contract = toml::from_str(&content).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;
    Ok(Some(contract))
}

/// List all contract IDs in the specs/contracts/ directory.
pub fn list_contracts(layout: &RepoLayout) -> ScaffoldResult<Vec<String>> {
    let dir = layout.contracts_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                ids.push(stem.to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_load_contract() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        std::fs::create_dir_all(layout.contracts_dir()).unwrap();

        let contract = Contract::new_draft(
            "kv-store".to_string(),
            "KV Store".to_string(),
            "Basic KV".to_string(),
        );
        write_contract(&layout, &contract).unwrap();

        let loaded = load_contract(&layout, "kv-store").unwrap().unwrap();
        assert_eq!(loaded.id, "kv-store");
        assert_eq!(loaded.title, "KV Store");
    }

    #[test]
    fn test_list_contracts() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        std::fs::create_dir_all(layout.contracts_dir()).unwrap();

        let c1 = Contract::new_draft("alpha".to_string(), "A".to_string(), "a".to_string());
        let c2 = Contract::new_draft("beta".to_string(), "B".to_string(), "b".to_string());
        write_contract(&layout, &c1).unwrap();
        write_contract(&layout, &c2).unwrap();

        let ids = list_contracts(&layout).unwrap();
        assert_eq!(ids, vec!["alpha", "beta"]);
    }
}
