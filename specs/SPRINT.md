# Lexicon Sprint 002 — Phase 2 Improvements

**Goal**: API extraction, contract coverage analysis, and stronger verification integration.

Previous sprint archived at `specs/sprints/001-initial-implementation.md`.

---

## Phase 12: New Crates — api + coverage

### 12a: `crates/api` — Public API Extraction
- [x] Create crate with workspace deps (syn, serde, serde_json)
- [x] `extract.rs` — parse Rust source files, extract public items (structs, enums, traits, functions, methods, modules, constants, types)
- [x] `schema.rs` — `ApiItem` type (kind, name, module_path, signature, visibility, trait_associations, stability, doc_summary)
- [x] `baseline.rs` — save/load baseline JSON (`.lexicon/api/baseline.json`)
- [x] `diff.rs` — compare current vs baseline (added, removed, changed signature/visibility/bounds/generics)
- [x] `report.rs` — human-readable + machine-readable diff report (breaking, additive, dangerous changes)
- [x] Tests (extraction, diffing, round-trip serialization)

### 12b: `crates/coverage` — Contract Coverage Analysis
- [x] Create crate with workspace deps
- [x] `analyzer.rs` — scan test files for `lexicon::tags(...)` or `#[lexicon_tag("...")]` attributes
- [x] `matcher.rs` — match test tags to contract clause `test_tags` fields
- [x] `report.rs` — compute coverage % per contract, list uncovered clauses
- [x] Tests

---

## Phase 13: Spec Extensions

- [x] Add `expected_api` field to Contract (list of expected traits/methods/types)
- [x] Add `test_tags` field to Invariant type (already on Semantic)
- [x] Add `contract_coverage` scoring dimension to default model
- [x] Add `api_drift` scoring dimension to default model
- [x] Add `ApiScan`, `ApiDiff`, `CoverageReport` audit actions
- [x] Add `api_dir()` path to RepoLayout (`.lexicon/api/`)
- [x] Validation for expected_api references
- [x] Tests for schema changes

---

## Phase 14: Core Integration

### 14a: API Commands
- [x] `core/api.rs` — `api_scan()`, `api_diff()`, `api_report()` orchestration
- [x] Write scan results to `.lexicon/api/current.json`
- [x] Baseline management (save current as baseline)
- [x] Audit records for API scan/diff

### 14b: Coverage Commands
- [x] `core/coverage.rs` — `coverage_report()` orchestration
- [x] Compute coverage per contract
- [x] Integrate coverage into scoring (contract_coverage dimension)

### 14c: Verify Pipeline Extension
- [x] Extend `verify()` to include API drift check
- [x] Extend `verify()` to include contract coverage
- [x] Score breakdown includes contract_coverage and api_drift dimensions
- [x] Verify output shows coverage report and API drift report

### 14d: Doctor Extension
- [x] Detect contract vs API mismatches (expected_api vs extracted API)
- [x] Detect uncovered contract clauses (no matching test tags)
- [x] Detect undocumented public API (not referenced by any contract)
- [x] Detect API drift from baseline

---

## Phase 15: CLI Commands

- [x] `lexicon api scan` — extract and store public API
- [x] `lexicon api diff` — compare current vs baseline
- [x] `lexicon api report` — summary with contract mismatch warnings
- [x] `lexicon api baseline` — save current as baseline
- [x] `lexicon coverage report` — contract coverage analysis
- [x] Update `lexicon verify` output to include coverage + API drift
- [x] Update `lexicon doctor` output to include new drift checks
- [x] Update `lexicon score explain` to show contract_coverage dimension

---

## Phase 16: TUI Updates

- [x] Add API tab to TUI (extracted API summary, drift status)
- [x] Add Coverage view (per-contract coverage %, uncovered clauses)
- [x] Update Dashboard to show API drift and coverage status

---

## Phase 17: Tests & Documentation

- [x] Integration tests for API scan -> diff -> report flow
- [x] Integration tests for coverage analysis flow
- [x] Integration test for extended verify pipeline
- [x] Snapshot tests for API diff output
- [x] Snapshot tests for coverage report
- [x] Update docs: new concept pages (API extraction, coverage)
- [x] Update docs: new command reference pages (api, coverage)
- [x] Update architecture doc with new crates
- [x] Update quickstart guide

---

## Phase 18: Authentication System

### 18a: Auth Types (`crates/spec/src/auth.rs`)
- [x] `Provider` enum (Claude, OpenAI) with `Display`, `FromStr`, serde
- [x] `ProviderConfig` struct (client_id, auth_url, token_url, port, scopes)
- [x] `Credentials` struct (provider, access_token, refresh_token, expires_at)
- [x] `Credentials::is_expired()` with 60s grace period
- [x] Unit tests for Provider round-trip, display, config validation, expiry

### 18b: Core Auth Module (`crates/core/src/auth.rs`)
- [x] PKCE OAuth flow (generate verifier/challenge, state, browser open, callback server)
- [x] `login(layout, provider, port_override)` — full OAuth flow with port fallback
- [x] `refresh(layout, provider)` — refresh expired tokens
- [x] `load(layout, provider)` / `save(layout, creds)` / `remove(layout, provider)`
- [x] `status(layout)` — check all providers
- [x] `ensure_authenticated(layout, provider)` — load + auto-refresh + error
- [x] Token exchange (JSON for Claude, form-encoded for OpenAI)
- [x] File permissions 0o600 on credential files (Unix)
- [x] Auth error variants in `CoreError`
- [x] Unit tests for PKCE, base64url, percent-encode, query-param, save/load/remove, status, permissions

### 18c: CLI Commands (`crates/cli/src/commands/auth.rs`)
- [x] `lexicon auth login` — interactive provider selection, browser OAuth
- [x] `lexicon auth refresh` — refresh expired tokens with spinner
- [x] `lexicon auth status` — show credential status for all providers
- [x] `lexicon auth logout` — remove stored credentials
- [x] Wire up `AuthAction` enum and match arm in `app.rs` / `main.rs`

### 18d: Infrastructure
- [x] Add workspace deps: `reqwest`, `sha2`, `open`
- [x] Add `auth_dir()` and `auth_credential_path()` to `RepoLayout`
- [x] Add `auth_dir()` to `init_dirs()`

### 18e: AI Feature Integration
- [x] `ensure_authenticated(claude)` call before `Improve` command

---

**Starting state**: 103 tests, 13 crates, 24-page docs site
**Ending state**: 167 tests, 15 crates, 28-page docs site
