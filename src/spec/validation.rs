use crate::spec::contract::Contract;
use crate::spec::error::{SpecError, SpecResult};
use crate::spec::manifest::Manifest;
use crate::spec::version::SchemaVersion;

/// Validate that a schema version is compatible with the current version.
pub fn validate_version(version: &SchemaVersion) -> SpecResult<()> {
    if !version.is_compatible_with(&SchemaVersion::CURRENT)
        && !SchemaVersion::CURRENT.is_compatible_with(version)
    {
        return Err(SpecError::IncompatibleVersion {
            found: version.clone(),
            expected: SchemaVersion::CURRENT,
        });
    }
    Ok(())
}

/// Convert a human-readable title into a kebab-case slug suitable as a contract ID.
///
/// Examples: "Key-Value Store" -> "key-value-store", "Rate Limiter (v2)" -> "rate-limiter-v2"
pub fn slugify(title: &str) -> String {
    let slug: String = title
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse multiple hyphens and trim leading/trailing
    let mut result = String::new();
    let mut prev_hyphen = true;
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    if result.ends_with('-') {
        result.pop();
    }
    result
}

/// Validate a contract id is a valid kebab-case slug.
pub fn validate_contract_id(id: &str) -> SpecResult<()> {
    if id.is_empty() {
        return Err(SpecError::InvalidContractId { id: id.to_string() });
    }
    // Must be lowercase alphanumeric with hyphens, no leading/trailing hyphens
    let valid = id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--");
    if !valid {
        return Err(SpecError::InvalidContractId { id: id.to_string() });
    }
    Ok(())
}

/// Validate a contract for structural correctness.
pub fn validate_contract(contract: &Contract) -> SpecResult<()> {
    validate_version(&contract.schema_version)?;
    validate_contract_id(&contract.id)?;

    if contract.title.trim().is_empty() {
        return Err(SpecError::MissingField {
            field: "title".to_string(),
        });
    }

    if contract.scope.trim().is_empty() {
        return Err(SpecError::MissingField {
            field: "scope".to_string(),
        });
    }

    // Check for duplicate invariant IDs
    let mut seen_ids = std::collections::HashSet::new();
    for inv in &contract.invariants {
        if !seen_ids.insert(&inv.id) {
            return Err(SpecError::DuplicateId {
                id: inv.id.clone(),
            });
        }
    }

    // Check for duplicate expected_api entries
    {
        let mut seen_api = std::collections::HashSet::new();
        for api_ref in &contract.expected_api {
            if !seen_api.insert(api_ref) {
                return Err(SpecError::DuplicateId {
                    id: api_ref.clone(),
                });
            }
        }
    }

    // Check for duplicate semantic IDs
    for sem in &contract.required_semantics {
        if !seen_ids.insert(&sem.id) {
            return Err(SpecError::DuplicateId {
                id: sem.id.clone(),
            });
        }
    }
    for sem in &contract.forbidden_semantics {
        if !seen_ids.insert(&sem.id) {
            return Err(SpecError::DuplicateId {
                id: sem.id.clone(),
            });
        }
    }

    Ok(())
}

/// Validate a manifest for structural correctness.
pub fn validate_manifest(manifest: &Manifest) -> SpecResult<()> {
    validate_version(&manifest.schema_version)?;

    if manifest.project.name.trim().is_empty() {
        return Err(SpecError::MissingField {
            field: "project.name".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::contract::Contract;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Key-Value Store"), "key-value-store");
        assert_eq!(slugify("My Awesome  Library!"), "my-awesome-library");
        assert_eq!(slugify("  Rate Limiter (v2) "), "rate-limiter-v2");
        assert_eq!(slugify("simple"), "simple");
        assert_eq!(slugify("UPPER CASE"), "upper-case");
        assert_eq!(slugify("dots.and_underscores"), "dots-and-underscores");
        // Slugified titles should pass contract ID validation
        assert!(validate_contract_id(&slugify("Key-Value Store")).is_ok());
        assert!(validate_contract_id(&slugify("Rate Limiter (v2)")).is_ok());
    }

    #[test]
    fn test_valid_contract_ids() {
        assert!(validate_contract_id("key-value-store").is_ok());
        assert!(validate_contract_id("parser").is_ok());
        assert!(validate_contract_id("my-lib-v2").is_ok());
        assert!(validate_contract_id("a").is_ok());
    }

    #[test]
    fn test_invalid_contract_ids() {
        assert!(validate_contract_id("").is_err());
        assert!(validate_contract_id("-leading").is_err());
        assert!(validate_contract_id("trailing-").is_err());
        assert!(validate_contract_id("double--dash").is_err());
        assert!(validate_contract_id("UPPERCASE").is_err());
        assert!(validate_contract_id("has spaces").is_err());
        assert!(validate_contract_id("has_underscores").is_err());
    }

    #[test]
    fn test_validate_contract_missing_title() {
        let contract = Contract::new_draft(
            "test".to_string(),
            "".to_string(),
            "scope".to_string(),
        );
        let result = validate_contract(&contract);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_contract_duplicate_ids() {
        let mut contract = Contract::new_draft(
            "test".to_string(),
            "Test".to_string(),
            "scope".to_string(),
        );
        contract.invariants.push(crate::spec::contract::Invariant {
            id: "dup".to_string(),
            description: "first".to_string(),
            severity: crate::spec::common::Severity::Required,
            test_tags: Vec::new(),
        });
        contract.invariants.push(crate::spec::contract::Invariant {
            id: "dup".to_string(),
            description: "second".to_string(),
            severity: crate::spec::common::Severity::Required,
            test_tags: Vec::new(),
        });
        let result = validate_contract(&contract);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_contract() {
        let contract = Contract::new_draft(
            "test".to_string(),
            "Test Contract".to_string(),
            "Test scope".to_string(),
        );
        assert!(validate_contract(&contract).is_ok());
    }
}
