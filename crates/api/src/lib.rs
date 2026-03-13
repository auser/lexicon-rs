//! Lexicon API crate — public API extraction, diffing, and baseline management.

pub mod baseline;
pub mod diff;
pub mod error;
pub mod extract;
pub mod report;
pub mod schema;

#[cfg(test)]
mod snapshot_tests;
