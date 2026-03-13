//! Lexicon ai crate — AI prompt/context generation and integration boundaries.

pub mod boundary;
pub mod context;
pub mod error;
pub mod policy;

#[cfg(test)]
mod snapshot_tests;
