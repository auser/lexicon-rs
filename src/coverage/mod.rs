//! Contract coverage analysis for lexicon.
//!
//! This crate scans test files for lexicon tags and matches them
//! against contract clauses to compute coverage metrics.

pub mod analyzer;
pub mod error;
pub mod matcher;
pub mod report;

#[cfg(test)]
mod snapshot_tests;
