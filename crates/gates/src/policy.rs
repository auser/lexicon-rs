use lexicon_spec::common::DimensionCategory;
use lexicon_spec::gates::Gate;

use crate::error::{GatesError, GatesResult};

/// Validate that skip requests do not violate policy.
///
/// Required gates cannot be skipped unless `allow_skip` is explicitly true.
pub fn validate_skip_request(gate: &Gate) -> GatesResult<()> {
    if gate.category == DimensionCategory::Required && !gate.allow_skip {
        return Err(GatesError::CannotSkipRequired {
            gate_id: gate.id.clone(),
        });
    }
    Ok(())
}

/// Check if a gate change (e.g., removing a gate or lowering category)
/// constitutes a weakening that requires approval.
pub fn is_weakening(old: &Gate, new: &Gate) -> bool {
    // Lowering category is weakening
    let old_strength = category_strength(old.category);
    let new_strength = category_strength(new.category);
    if new_strength < old_strength {
        return true;
    }

    // Making a non-skippable gate skippable is weakening
    if !old.allow_skip && new.allow_skip {
        return true;
    }

    false
}

fn category_strength(cat: DimensionCategory) -> u8 {
    match cat {
        DimensionCategory::Required => 3,
        DimensionCategory::Scored => 2,
        DimensionCategory::Advisory => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_gate(id: &str, cat: DimensionCategory, allow_skip: bool) -> Gate {
        Gate {
            id: id.to_string(),
            label: id.to_string(),
            command: "true".to_string(),
            category: cat,
            timeout_secs: None,
            allow_skip,
        }
    }

    #[test]
    fn test_cannot_skip_required() {
        let gate = make_gate("fmt", DimensionCategory::Required, false);
        assert!(validate_skip_request(&gate).is_err());
    }

    #[test]
    fn test_can_skip_advisory() {
        let gate = make_gate("docs", DimensionCategory::Advisory, true);
        assert!(validate_skip_request(&gate).is_ok());
    }

    #[test]
    fn test_weakening_category() {
        let old = make_gate("test", DimensionCategory::Required, false);
        let new = make_gate("test", DimensionCategory::Advisory, false);
        assert!(is_weakening(&old, &new));
    }

    #[test]
    fn test_weakening_skip() {
        let old = make_gate("test", DimensionCategory::Required, false);
        let new = make_gate("test", DimensionCategory::Required, true);
        assert!(is_weakening(&old, &new));
    }

    #[test]
    fn test_not_weakening() {
        let old = make_gate("test", DimensionCategory::Scored, true);
        let new = make_gate("test", DimensionCategory::Required, false);
        assert!(!is_weakening(&old, &new));
    }
}
