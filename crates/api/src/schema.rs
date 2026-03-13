use serde::{Deserialize, Serialize};

/// The kind of API item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ApiItemKind {
    Struct,
    Enum,
    Trait,
    Function,
    Method,
    Module,
    Constant,
    TypeAlias,
    Impl,
}

impl std::fmt::Display for ApiItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Trait => write!(f, "trait"),
            Self::Function => write!(f, "function"),
            Self::Method => write!(f, "method"),
            Self::Module => write!(f, "module"),
            Self::Constant => write!(f, "constant"),
            Self::TypeAlias => write!(f, "type alias"),
            Self::Impl => write!(f, "impl"),
        }
    }
}

/// Visibility level of an API item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Crate,
    Restricted,
    Private,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "pub"),
            Self::Crate => write!(f, "pub(crate)"),
            Self::Restricted => write!(f, "pub(restricted)"),
            Self::Private => write!(f, "private"),
        }
    }
}

/// A single public API item extracted from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiItem {
    pub kind: ApiItemKind,
    pub name: String,
    pub module_path: Vec<String>,
    pub signature: String,
    pub visibility: Visibility,
    pub trait_associations: Vec<String>,
    pub stability: Option<String>,
    pub doc_summary: Option<String>,
    pub span_file: Option<String>,
    pub span_line: Option<u32>,
}

impl PartialEq for ApiItem {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.name == other.name
            && self.module_path == other.module_path
            && self.signature == other.signature
            && self.visibility == other.visibility
    }
}

impl Eq for ApiItem {}

/// A snapshot of all public API items in a crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSnapshot {
    pub crate_name: String,
    pub version: Option<String>,
    pub items: Vec<ApiItem>,
    pub extracted_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_item() -> ApiItem {
        ApiItem {
            kind: ApiItemKind::Function,
            name: "do_thing".into(),
            module_path: vec!["mymod".into()],
            signature: "fn do_thing(x: i32) -> bool".into(),
            visibility: Visibility::Public,
            trait_associations: vec![],
            stability: None,
            doc_summary: Some("Does a thing.".into()),
            span_file: Some("src/lib.rs".into()),
            span_line: Some(10),
        }
    }

    #[test]
    fn serde_roundtrip_api_item() {
        let item = sample_item();
        let json = serde_json::to_string(&item).unwrap();
        let back: ApiItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, back);
    }

    #[test]
    fn serde_roundtrip_api_snapshot() {
        let snap = ApiSnapshot {
            crate_name: "my-crate".into(),
            version: Some("0.1.0".into()),
            items: vec![sample_item()],
            extracted_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string_pretty(&snap).unwrap();
        let back: ApiSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap.crate_name, back.crate_name);
        assert_eq!(snap.items.len(), back.items.len());
        assert_eq!(snap.items[0], back.items[0]);
    }
}
