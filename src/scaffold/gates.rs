use crate::repo::layout::RepoLayout;
use crate::spec::gates::GatesModel;

use super::error::ScaffoldResult;

/// Write the gates model to specs/gates.toml.
pub fn write_gates_model(layout: &RepoLayout, model: &GatesModel) -> ScaffoldResult<()> {
    let path = layout.gates_path();
    let content = toml::to_string_pretty(model)?;
    crate::fs::safe_write::safe_write(&path, &content, false)?;
    Ok(())
}

/// Load the gates model.
pub fn load_gates_model(layout: &RepoLayout) -> ScaffoldResult<Option<GatesModel>> {
    let path = layout.gates_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let model: GatesModel = toml::from_str(&content).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;
    Ok(Some(model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_load_gates_model() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        std::fs::create_dir_all(layout.specs_dir()).unwrap();

        let model = GatesModel::default_model();
        write_gates_model(&layout, &model).unwrap();

        let loaded = load_gates_model(&layout).unwrap().unwrap();
        assert_eq!(loaded.gates.len(), model.gates.len());
    }
}
