//! Lexicon ai crate — AI prompt/context generation and integration boundaries.

pub mod boundary;
pub mod client;
pub mod context;
pub mod error;
pub mod generate;
pub mod policy;
pub mod prompt;

#[cfg(test)]
mod snapshot_tests;
