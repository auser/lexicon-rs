use lexicon_repo::layout::RepoLayout;
use lexicon_spec::gates::GatesModel;
use lexicon_spec::scoring::ScoreModel;

use crate::error::CoreResult;

/// Initialize the default scoring model.
pub fn score_init(layout: &RepoLayout) -> CoreResult<()> {
    let model = ScoreModel::default_model();
    lexicon_scaffold::scoring::write_score_model(layout, &model)?;
    Ok(())
}

/// Initialize the default gates model.
pub fn gate_init(layout: &RepoLayout) -> CoreResult<()> {
    let model = GatesModel::default_model();
    lexicon_scaffold::gates::write_gates_model(layout, &model)?;
    Ok(())
}

/// Explain the current scoring model.
pub fn score_explain(layout: &RepoLayout) -> CoreResult<String> {
    let model = lexicon_scaffold::scoring::load_score_model(layout)?;
    match model {
        Some(model) => {
            let mut lines = Vec::new();
            lines.push("Scoring Model".to_string());
            lines.push(format!(
                "Pass: {:.0}%, Warn: {:.0}%",
                model.thresholds.pass * 100.0,
                model.thresholds.warn * 100.0
            ));
            lines.push(String::new());
            let total_weight: u32 = model.dimensions.iter().map(|d| d.weight).sum();
            for dim in &model.dimensions {
                lines.push(format!(
                    "  {} (weight: {}/{}, {:?}, source: {:?})",
                    dim.label, dim.weight, total_weight, dim.category, dim.source
                ));
            }
            Ok(lines.join("\n"))
        }
        None => Ok("No scoring model configured. Run `lexicon score init` first.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_score_and_gate_init() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        crate::init::init_repo_noninteractive(
            &layout,
            "test".to_string(),
            "test".to_string(),
            lexicon_spec::common::RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();

        score_init(&layout).unwrap();
        gate_init(&layout).unwrap();

        assert!(layout.scoring_model_path().exists());
        assert!(layout.gates_path().exists());
    }

    #[test]
    fn test_score_explain() {
        let dir = TempDir::new().unwrap();
        let layout = RepoLayout::new(dir.path().to_path_buf());
        crate::init::init_repo_noninteractive(
            &layout,
            "test".to_string(),
            "test".to_string(),
            lexicon_spec::common::RepoType::Library,
            "testing".to_string(),
        )
        .unwrap();

        // Before init
        let explanation = score_explain(&layout).unwrap();
        assert!(explanation.contains("No scoring model"));

        // After init
        score_init(&layout).unwrap();
        let explanation = score_explain(&layout).unwrap();
        assert!(explanation.contains("Scoring Model"));
        assert!(explanation.contains("Correctness"));
    }
}
