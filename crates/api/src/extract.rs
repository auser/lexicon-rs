use std::path::Path;

use syn::visit::Visit;

use crate::error::ApiError;
use crate::schema::{ApiItem, ApiItemKind, ApiSnapshot, Visibility};

/// Extract public API items from a Rust source string.
pub fn extract_from_source(source: &str, file_path: &str) -> Result<Vec<ApiItem>, ApiError> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = ApiVisitor {
        items: Vec::new(),
        module_path: Vec::new(),
        file_path: file_path.to_string(),
    };
    visitor.visit_file(&syntax);
    Ok(visitor.items)
}

/// Extract public API items from a Rust source file.
pub fn extract_from_file(path: &Path) -> Result<Vec<ApiItem>, ApiError> {
    let source = std::fs::read_to_string(path)?;
    let file_path = path.to_string_lossy().to_string();
    extract_from_source(&source, &file_path)
}

/// Extract a full API snapshot from a directory of Rust source files.
pub fn extract_from_dir(dir: &Path) -> Result<ApiSnapshot, ApiError> {
    let mut all_items = Vec::new();
    walk_rs_files(dir, &mut all_items)?;
    Ok(ApiSnapshot {
        crate_name: dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        version: None,
        items: all_items,
        extracted_at: chrono_now(),
    })
}

fn walk_rs_files(dir: &Path, items: &mut Vec<ApiItem>) -> Result<(), ApiError> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, items)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            match extract_from_file(&path) {
                Ok(mut file_items) => items.append(&mut file_items),
                Err(ApiError::Parse(_)) => {
                    // Skip files that fail to parse
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok(())
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without pulling in chrono
    // We use a fixed format for reproducibility in tests
    let now = std::time::SystemTime::now();
    let dur = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Simple formatting: just use the unix timestamp in a readable way
    format!("{secs}")
}

fn convert_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility::Public,
        syn::Visibility::Restricted(r) => {
            let path_str = r.path.segments.iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if path_str == "crate" {
                Visibility::Crate
            } else if path_str == "super" || path_str.contains("in") {
                Visibility::Restricted
            } else {
                Visibility::Restricted
            }
        }
        syn::Visibility::Inherited => Visibility::Private,
    }
}

fn extract_doc_summary(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let syn::Lit::Str(s) = &expr_lit.lit {
                        let text = s.value();
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn format_fn_signature(sig: &syn::Signature) -> String {
    let unsafety = if sig.unsafety.is_some() { "unsafe " } else { "" };
    let asyncness = if sig.asyncness.is_some() { "async " } else { "" };
    let ident = &sig.ident;

    let generics = if sig.generics.params.is_empty() {
        String::new()
    } else {
        let params: Vec<String> = sig.generics.params.iter().map(|p| {
            quote_to_string(p)
        }).collect();
        format!("<{}>", params.join(", "))
    };

    let inputs: Vec<String> = sig.inputs.iter().map(|arg| {
        quote_to_string(arg)
    }).collect();

    let output = match &sig.output {
        syn::ReturnType::Default => String::new(),
        syn::ReturnType::Type(_, ty) => format!(" -> {}", quote_to_string(ty)),
    };

    let where_clause = sig.generics.where_clause.as_ref().map(|w| {
        format!(" {}", quote_to_string(w))
    }).unwrap_or_default();

    format!("{asyncness}{unsafety}fn {ident}{generics}({inputs}){output}{where_clause}",
        inputs = inputs.join(", "))
}

fn quote_to_string(tokens: &dyn quote::ToTokens) -> String {
    let ts = quote::quote!(#tokens);
    ts.to_string()
}

fn format_generics(generics: &syn::Generics) -> String {
    if generics.params.is_empty() {
        return String::new();
    }
    let params: Vec<String> = generics.params.iter().map(|p| quote_to_string(p)).collect();
    format!("<{}>", params.join(", "))
}

fn format_supertraits(supertraits: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>) -> String {
    if supertraits.is_empty() {
        return String::new();
    }
    let bounds: Vec<String> = supertraits.iter().map(|b| quote_to_string(b)).collect();
    format!(": {}", bounds.join(" + "))
}

struct ApiVisitor {
    items: Vec<ApiItem>,
    module_path: Vec<String>,
    file_path: String,
}

impl<'ast> Visit<'ast> for ApiVisitor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            let generics = format_generics(&node.generics);
            self.items.push(ApiItem {
                kind: ApiItemKind::Struct,
                name: node.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format!("struct {}{generics}", node.ident),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.ident.span().start().line as u32),
            });
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            let generics = format_generics(&node.generics);
            self.items.push(ApiItem {
                kind: ApiItemKind::Enum,
                name: node.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format!("enum {}{generics}", node.ident),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.ident.span().start().line as u32),
            });
        }
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            let generics = format_generics(&node.generics);
            let supers = format_supertraits(&node.supertraits);
            self.items.push(ApiItem {
                kind: ApiItemKind::Trait,
                name: node.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format!("trait {}{generics}{supers}", node.ident),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.ident.span().start().line as u32),
            });
        }
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            self.items.push(ApiItem {
                kind: ApiItemKind::Function,
                name: node.sig.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format_fn_signature(&node.sig),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.sig.ident.span().start().line as u32),
            });
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            let ty = quote_to_string(&node.ty);
            self.items.push(ApiItem {
                kind: ApiItemKind::Constant,
                name: node.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format!("const {}: {ty}", node.ident),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.ident.span().start().line as u32),
            });
        }
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            let generics = format_generics(&node.generics);
            let ty = quote_to_string(&node.ty);
            self.items.push(ApiItem {
                kind: ApiItemKind::TypeAlias,
                name: node.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format!("type {}{generics} = {ty}", node.ident),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.ident.span().start().line as u32),
            });
        }
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let vis = convert_visibility(&node.vis);
        if matches!(vis, Visibility::Public | Visibility::Crate) {
            self.items.push(ApiItem {
                kind: ApiItemKind::Module,
                name: node.ident.to_string(),
                module_path: self.module_path.clone(),
                signature: format!("mod {}", node.ident),
                visibility: vis,
                trait_associations: vec![],
                stability: None,
                doc_summary: extract_doc_summary(&node.attrs),
                span_file: Some(self.file_path.clone()),
                span_line: Some(node.ident.span().start().line as u32),
            });
        }
        // Recurse into module body
        self.module_path.push(node.ident.to_string());
        syn::visit::visit_item_mod(self, node);
        self.module_path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_pub_struct() {
        let source = r#"
            pub struct Foo {
                pub x: i32,
            }
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ApiItemKind::Struct);
        assert_eq!(items[0].name, "Foo");
        assert_eq!(items[0].signature, "struct Foo");
        assert_eq!(items[0].visibility, Visibility::Public);
    }

    #[test]
    fn extract_pub_enum() {
        let source = r#"
            pub enum Color {
                Red,
                Green,
                Blue,
            }
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ApiItemKind::Enum);
        assert_eq!(items[0].name, "Color");
        assert_eq!(items[0].signature, "enum Color");
    }

    #[test]
    fn extract_pub_trait() {
        let source = r#"
            pub trait Drawable: Clone {
                fn draw(&self);
            }
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ApiItemKind::Trait);
        assert_eq!(items[0].name, "Drawable");
        assert!(items[0].signature.contains("trait Drawable"));
        assert!(items[0].signature.contains("Clone"));
    }

    #[test]
    fn extract_pub_function() {
        let source = r#"
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ApiItemKind::Function);
        assert_eq!(items[0].name, "add");
        assert!(items[0].signature.contains("fn add"));
        assert!(items[0].signature.contains("i32"));
    }

    #[test]
    fn extract_pub_const() {
        let source = r#"
            pub const MAX: u32 = 100;
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ApiItemKind::Constant);
        assert_eq!(items[0].name, "MAX");
    }

    #[test]
    fn extract_pub_type_alias() {
        let source = r#"
            pub type Result<T> = std::result::Result<T, MyError>;
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ApiItemKind::TypeAlias);
        assert_eq!(items[0].name, "Result");
    }

    #[test]
    fn skip_private_items() {
        let source = r#"
            struct Private;
            fn private_fn() {}
            pub struct Public;
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Public");
    }

    #[test]
    fn extract_doc_comment() {
        let source = r#"
            /// Does something useful.
            /// More details here.
            pub fn useful() {}
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].doc_summary.as_deref(), Some("Does something useful."));
    }

    #[test]
    fn extract_pub_crate() {
        let source = r#"
            pub(crate) fn internal() {}
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].visibility, Visibility::Crate);
    }

    #[test]
    fn extract_nested_module() {
        let source = r#"
            pub mod outer {
                pub fn inner_fn() {}
            }
        "#;
        let items = extract_from_source(source, "test.rs").unwrap();
        // Should have the module and the function inside it
        assert_eq!(items.len(), 2);
        let func = items.iter().find(|i| i.kind == ApiItemKind::Function).unwrap();
        assert_eq!(func.module_path, vec!["outer".to_string()]);
    }
}
