# Lexicon Sprint 002 — Phase 2 Improvements

**Goal**: API extraction, contract coverage analysis, and stronger verification integration.

Previous sprint archived at `specs/sprints/001-initial-implementation.md`.

---

## Phase 12: New Crates — api + coverage

### 12a: `crates/api` — Public API Extraction
- [ ] Create crate with workspace deps (syn, serde, serde_json)
- [ ] `extract.rs` — parse Rust source files, extract public items (structs, enums, traits, functions, methods, modules, constants, types)
- [ ] `schema.rs` — `ApiItem` type (kind, name, module_path, signature, visibility, trait_associations, stability, doc_summary)
- [ ] `baseline.rs` — save/load baseline JSON (`.lexicon/api/baseline.json`)
- [ ] `diff.rs` — compare current vs baseline (added, removed, changed signature/visibility/bounds/generics)
- [ ] `report.rs` — human-readable + machine-readable diff report (breaking, additive, dangerous changes)
- [ ] Tests (extraction, diffing, round-trip serialization)

### 12b: `crates/coverage` — Contract Coverage Analysis
- [ ] Create crate with workspace deps
- [ ] `analyzer.rs` — scan test files for `lexicon::tags(...)` or `#[lexicon_tag("...")]` attributes
- [ ] `matcher.rs` — match test tags to contract clause `test_tags` fields
- [ ] `report.rs` — compute coverage % per contract, list uncovered clauses
- [ ] Tests

---

## Phase 13: Spec Extensions

- [ ] Add `expected_api` field to Contract (list of expected traits/methods/types)
- [ ] Add `test_tags` field to Invariant type (already on Semantic)
- [ ] Add `contract_coverage` scoring dimension to default model
- [ ] Add `api_drift` scoring dimension to default model
- [ ] Add `ApiScan`, `ApiDiff`, `CoverageReport` audit actions
- [ ] Add `api_dir()` path to RepoLayout (`.lexicon/api/`)
- [ ] Validation for expected_api references
- [ ] Tests for schema changes

---

## Phase 14: Core Integration

### 14a: API Commands
- [ ] `core/api.rs` — `api_scan()`, `api_diff()`, `api_report()` orchestration
- [ ] Write scan results to `.lexicon/api/current.json`
- [ ] Baseline management (save current as baseline)
- [ ] Audit records for API scan/diff

### 14b: Coverage Commands
- [ ] `core/coverage.rs` — `coverage_report()` orchestration
- [ ] Compute coverage per contract
- [ ] Integrate coverage into scoring (contract_coverage dimension)

### 14c: Verify Pipeline Extension
- [ ] Extend `verify()` to include API drift check
- [ ] Extend `verify()` to include contract coverage
- [ ] Score breakdown includes contract_coverage and api_drift dimensions
- [ ] Verify output shows coverage report and API drift report

### 14d: Doctor Extension
- [ ] Detect contract vs API mismatches (expected_api vs extracted API)
- [ ] Detect uncovered contract clauses (no matching test tags)
- [ ] Detect undocumented public API (not referenced by any contract)
- [ ] Detect API drift from baseline

---

## Phase 15: CLI Commands

- [ ] `lexicon api scan` — extract and store public API
- [ ] `lexicon api diff` — compare current vs baseline
- [ ] `lexicon api report` — summary with contract mismatch warnings
- [ ] `lexicon api baseline` — save current as baseline
- [ ] `lexicon coverage report` — contract coverage analysis
- [ ] Update `lexicon verify` output to include coverage + API drift
- [ ] Update `lexicon doctor` output to include new drift checks
- [ ] Update `lexicon score explain` to show contract_coverage dimension

---

## Phase 16: TUI Updates

- [ ] Add API tab to TUI (extracted API summary, drift status)
- [ ] Add Coverage view (per-contract coverage %, uncovered clauses)
- [ ] Update Dashboard to show API drift and coverage status

---

## Phase 17: Tests & Documentation

- [ ] Integration tests for API scan -> diff -> report flow
- [ ] Integration tests for coverage analysis flow
- [ ] Integration test for extended verify pipeline
- [ ] Snapshot tests for API diff output
- [ ] Snapshot tests for coverage report
- [ ] Update docs: new concept pages (API extraction, coverage)
- [ ] Update docs: new command reference pages (api, coverage)
- [ ] Update architecture doc with new crates
- [ ] Update quickstart guide

---

**Starting state**: 103 tests, 13 crates, 24-page docs site
