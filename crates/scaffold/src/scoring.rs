use lexicon_repo::layout::RepoLayout;
use lexicon_spec::scoring::ScoreModel;

use crate::error::ScaffoldResult;

/// Write the scoring model to specs/scoring/model.toml.
pub fn write_score_model(layout: &RepoLayout, model: &ScoreModel) -> ScaffoldResult<()> {
    let path = layout.scoring_model_path();
    let content = toml::to_string_pretty(model)?;
    lexicon_fs::safe_write::safe_write(&path, &content, false)?;
    Ok(())
}

/// Load the scoring model.
pub fn load_score_model(layout: &RepoLayout) -> ScaffoldResult<Option<ScoreModel>> {
    let path = layout.scoring_model_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let model: ScoreModel = toml::from_str(&content).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;
    Ok(Some(model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_load_score_model() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        std::fs::create_dir_all(layout.scoring_dir()).unwrap();

        let model = ScoreModel::default_model();
        write_score_model(&layout, &model).unwrap();

        let loaded = load_score_model(&layout).unwrap().unwrap();
        assert_eq!(loaded.dimensions.len(), model.dimensions.len());
    }
}
