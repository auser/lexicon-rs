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

## Phase 19: AI-Assisted Artifact Creation & Intent-Driven Generation

Implements specs `009-ai-assisted-work.md` and `010-ai-prompting.md`.

### 19a: Claude API Client (`crates/ai/src/client.rs`)
- [x] `ClaudeClient` struct implementing `AiProvider` trait
- [x] `complete(system, user_message)` — call Claude Messages API with OAuth Bearer token
- [x] Uses stored credentials from `lexicon auth login` (auto-refresh via `ensure_authenticated`)
- [x] Configurable model (defaults to claude-sonnet-4)
- [x] Response parsing (content blocks → text extraction)
- [x] Unit tests for client creation and model override

### 19b: Prompt Builder (`crates/ai/src/prompt.rs`)
- [x] `ArtifactKind` enum (Contract, Conformance, Behavior, Improve)
- [x] `system_prompt(kind)` — specialized system prompts per artifact type
- [x] `intent_prompt(kind, intent, context)` — user message with repo context + templates
- [x] `improve_prompt(context, artifacts, goal)` — improvement suggestion prompts
- [x] Templates for contract (TOML), conformance (Rust), behavior (Markdown)
- [x] Unit tests for prompt generation

### 19c: Artifact Generation Engine (`crates/ai/src/generate.rs`)
- [x] `GeneratedArtifact` struct (kind, path, content, format)
- [x] `generate_artifact(provider, layout, kind, intent)` — full generation pipeline
- [x] `generate_improvements(provider, layout, goal)` — improvement suggestions
- [x] Context assembly from repo state (manifest, contracts, scoring, gates)
- [x] Intent-to-slug path generation
- [x] Unit tests for slugify

### 19d: Core Integration (`crates/core/src/generate.rs`)
- [x] `generate_from_intent(layout, kind, intent)` — auth + AI + generate
- [x] `generate_improve(layout, goal)` — auth + AI + suggest improvements
- [x] `accept_artifact(layout, artifact)` — write to disk + audit record
- [x] `reject_artifact(layout, artifact)` — audit record for rejected suggestions
- [x] `build_ai_provider(layout)` — authenticate and build ClaudeClient

### 19e: CLI Commands
- [x] `lexicon generate "<intent>"` — generate contract from description
- [x] `lexicon contract generate "<intent>"` — AI-generate a contract
- [x] `lexicon conformance generate "<intent>"` — AI-generate conformance tests
- [x] `lexicon behavior generate "<intent>"` — AI-generate behavior scenarios
- [x] `lexicon improve [--goal <goal>]` — AI-guided improvement suggestions
- [x] Patch preview (content display + accept/reject flow)
- [x] Audit trail for accepted/rejected artifacts

### 19f: CLI Restructuring
- [x] New `commands/conformance.rs` — conformance subcommand dispatch
- [x] New `commands/behavior.rs` — behavior subcommand dispatch
- [x] New `commands/generate.rs` — generate + improve command handlers
- [x] Updated `app.rs` with Generate variants on Contract, Conformance, Behavior
- [x] Updated `main.rs` to dispatch all new commands

---

**Sprint 002 state**: 103→174 tests, 13→15 crates, 24→28-page docs site

---
---

# Lexicon Sprint 003 — Progressive Adoption, Docs & Polish

**Goal**: Progressive scale modes, docs diagrams, polished homepage, and documentation updates.

Implements specs `004-progressive-adoption.md`, `007-docs-diagrams.md`, `008-homepage.md`.

---

## Phase 20: Progressive Adoption — Operating Modes

Implements `specs/prompts/004-progressive-adoption.md`.

### 20a: Mode Detection & Capability Model (`crates/spec`)
- [ ] `mode.rs` — `OperatingMode` enum (Repo, Workspace, Ecosystem)
- [ ] `capability.rs` — `Capability` enum and `CapabilitySet` (repo_contracts, repo_conformance, repo_scoring, repo_gates, repo_api, workspace_architecture, workspace_dependency_law, workspace_shared_contracts, ecosystem_governance, ecosystem_shared_contracts, ecosystem_impact)
- [ ] `workspace.rs` — `WorkspaceManifest` schema type
- [ ] `ecosystem.rs` — `EcosystemManifest`, `RepoEntry` schema types
- [ ] Tests for mode detection, capability sets

### 20b: Repo Shape Detection (`crates/repo`)
- [ ] `detect.rs` — detect repo shape (single crate, workspace, ecosystem)
- [ ] `mode.rs` — `detect_mode(layout)` — auto-detect operating mode from repo structure
- [ ] Integration with `RepoLayout` for mode-aware paths
- [ ] Tests for shape detection

### 20c: Schema Layering
- [ ] `.lexicon/workspace.toml` — workspace manifest schema
- [ ] `.lexicon/architecture/rules.toml` — architecture rules schema
- [ ] `.lexicon/architecture/graph.json` — architecture graph schema
- [ ] `.lexicon/ecosystem.toml` — ecosystem manifest schema
- [ ] `.lexicon/ecosystem/repos.toml` — repo registry schema
- [ ] `.lexicon/ecosystem/contracts/*.toml` — shared contracts schema
- [ ] Validation for each schema layer

### 20d: Mode-Aware Init Flow
- [ ] `lexicon init` auto-detects repo shape and defaults to Repo Mode
- [ ] `lexicon workspace init` — upgrade from Repo Mode, preserve existing state
- [ ] `lexicon ecosystem init` — upgrade from Workspace Mode, preserve existing state
- [ ] Interactive prompts explaining mode options
- [ ] Migration safety (additive, no data loss)

### 20e: Mode-Aware Commands
- [ ] `lexicon workspace verify` — workspace-level verification
- [ ] `lexicon workspace doctor` — workspace-level health checks
- [ ] `lexicon ecosystem verify` — ecosystem-level verification
- [ ] `lexicon ecosystem doctor` — ecosystem-level health checks
- [ ] `lexicon verify` adapts output based on current mode
- [ ] `lexicon doctor` adapts output based on current mode

### 20f: Architecture Governance (`crates/architecture` — new crate)
- [ ] Create `crates/architecture` with workspace deps
- [ ] `roles.rs` — crate roles (foundation, interface, adapter, application)
- [ ] `rules.rs` — dependency law, layering rules
- [ ] `graph.rs` — architecture graph construction
- [ ] `drift.rs` — architecture drift detection
- [ ] `lexicon architecture graph` command

### 20g: Mode-Aware TUI
- [ ] Dashboard shows current mode badge (Repo/Workspace/Ecosystem)
- [ ] Workspace Mode: add crate role browser, architecture graph, dependency law explorer
- [ ] Ecosystem Mode: add repo role browser, ecosystem governance dashboard
- [ ] Tabs appear/disappear based on active mode

### 20h: Mode-Aware Conversation Loops
- [ ] Conversations scope questions to current mode
- [ ] Repo Mode: local public API, contract semantics, local test expectations
- [ ] Workspace Mode: additionally crate roles, crate boundaries, dependency direction
- [ ] Ecosystem Mode: additionally repo roles, cross-repo responsibilities, interface ownership

### 20i: Tests & Documentation
- [ ] Tests for repo shape detection
- [ ] Tests for mode selection
- [ ] Tests for init behavior by mode
- [ ] Tests for migration (repo → workspace, workspace → ecosystem)
- [ ] Tests for mode-aware verify and doctor
- [ ] Tests for schema layering
- [ ] Docs: "Progressive Adoption" concept page
- [ ] Docs: workspace and ecosystem command reference pages
- [ ] Update architecture doc with new crate

---

## Phase 21: Documentation Diagrams

Implements `specs/prompts/007-docs-diagrams.md`.

### 21a: Diagram Components
- [ ] `docs/src/components/diagrams/LexiconModelDiagram.astro` — core concept relationships
- [ ] `docs/src/components/diagrams/VerificationPipelineDiagram.astro` — verification flow
- [ ] `docs/src/components/diagrams/ProgressiveScopeDiagram.astro` — Repo → Workspace → Ecosystem
- [ ] `docs/src/components/diagrams/AISafetyDiagram.astro` — AI boundary model
- [ ] `docs/src/components/diagrams/ArchitectureGovernanceDiagram.astro` — architecture rules
- [ ] `docs/src/components/diagrams/ContractCoverageDiagram.astro` — coverage mapping

### 21b: Diagram Design
- [ ] Inline SVG, theme-aware (light/dark)
- [ ] Responsive layout, mobile legible
- [ ] Consistent visual style (soft gradients, rounded containers, clean typography)
- [ ] Reusable wrapper/caption components

### 21c: Docs Integration
- [ ] Lexicon model diagram on homepage or core concepts page
- [ ] Verification pipeline in verification docs
- [ ] Progressive scope diagram in getting started or scope docs
- [ ] AI safety diagram in AI agents docs
- [ ] Architecture governance diagram in architecture docs
- [ ] Contract coverage diagram in coverage docs

---

## Phase 22: Polished Homepage

Implements `specs/prompts/008-homepage.md`.

### 22a: Homepage Structure
- [ ] Custom hero section (title, value proposition, CTAs)
- [ ] Visual concept section with embedded diagram
- [ ] "Why Lexicon Exists" problem statement section
- [ ] Core concept highlights (feature grid: Contracts, Conformance, Coverage, Gates, Scoring, Architecture, Ecosystem, AI Context)
- [ ] Progressive scope section (Repo → Workspace → Ecosystem visual)
- [ ] AI safety section
- [ ] Quick start / calls to action section

### 22b: Implementation
- [ ] Custom `docs/src/content/docs/index.mdx` homepage
- [ ] Supporting Astro components (hero, feature card grid, CTA row, section wrappers)
- [ ] Responsive layout
- [ ] Starlight theme integration
- [ ] Strong, concise copy (no buzzwords, no hype)

---

## Phase 23: Remaining Polish & Fixes

### 23a: Misc
- [ ] Update `AGENTS.md` to include requirement to keep docs site up-to-date (from `specs/scratch.md`)
- [ ] Update ending state counts in SPRINT.md

---

## Backlog (Not Yet Scheduled)

These are referenced in spec prompts but not yet broken into phases:

- **Ecosystem Governance** (`crates/ecosystem` — new crate): federated contracts, repo roles, cross-repo compatibility, ecosystem-level architecture policies. Described in `006-project-description.md` and `004-progressive-adoption.md`.
- **Workspace Shared Contracts**: shared contract registry within a workspace, contract inheritance/composition.
- **Dependency Law**: explicit crate dependency rules, forbidden dependency detection, layer violation checks.
- **Impact Analysis**: downstream impact analysis for changes (workspace and ecosystem scope).
- **AI Prompt Refinement**: conversation storage for artifact generation sessions, session replay/refinement, prompt training data collection. Extended from `009-ai-assisted-work.md`.
- **Targeted Improve Commands**: `lexicon contract improve`, `lexicon conformance improve`, `lexicon coverage improve`, `lexicon architecture improve`, `lexicon scoring improve`. From `009-ai-assisted-work.md`.
- **TUI Artifact Review**: AI suggestions as reviewable patches in TUI mode. From `009-ai-assisted-work.md` and `010-ai-prompting.md`.
