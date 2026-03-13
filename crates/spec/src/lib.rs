//! Lexicon spec crate — domain types, schemas, and validation.
//!
//! This is the foundation crate for lexicon. All domain types,
//! schema definitions, and validation logic live here. Every other
//! crate in the workspace depends on this one.

pub mod auth;
pub mod audit;
pub mod behavior;
pub mod common;
pub mod conformance;
pub mod contract;
pub mod error;
pub mod gates;
pub mod manifest;
pub mod scoring;
pub mod session;
pub mod validation;
pub mod version;

#[cfg(test)]
mod snapshot_tests;
