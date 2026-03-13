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
- [x] `mode.rs` — `OperatingMode` enum (Repo, Workspace, Ecosystem) with PartialOrd/Ord
- [x] `capability.rs` — `Capability` enum and `CapabilitySet` with `for_mode()`, `has()`, `is_empty()`
- [x] `workspace.rs` — `WorkspaceManifest`, `CrateRole`, `CrateRoleKind`, `DependencyRule`
- [x] `ecosystem.rs` — `EcosystemManifest`, `RepoEntry`, `RepoRole`
- [x] Tests for mode detection, capability sets (17 tests across mode/capability/workspace/ecosystem)

### 20b: Repo Shape Detection (`crates/repo`)
- [x] `detect.rs` — `RepoShape` enum, `detect_shape(root)`, `detect_mode(layout)`
- [x] Auto-detect operating mode from `.lexicon/` files and Cargo.toml structure
- [x] Integration with `RepoLayout` for mode-aware paths (workspace_manifest_path, ecosystem_manifest_path, architecture_dir, etc.)
- [x] Tests for shape detection (7 tests)

### 20c: Schema Layering
- [x] `.lexicon/workspace.toml` — workspace manifest schema (WorkspaceManifest)
- [x] `.lexicon/architecture/rules.toml` — architecture rules schema (DependencyRule)
- [x] `.lexicon/architecture/graph.json` — architecture graph path in layout
- [x] `.lexicon/ecosystem.toml` — ecosystem manifest schema (EcosystemManifest)
- [x] `.lexicon/ecosystem/` — ecosystem directory for shared state
- [x] Serialization/deserialization for all schema types

### 20d: Mode-Aware Init Flow
- [x] `lexicon init` auto-detects repo shape and prints workspace hint
- [x] `lexicon workspace init` — upgrade from Repo Mode with auto-detected crate roles
- [x] `lexicon ecosystem init` — upgrade from Workspace Mode with ecosystem manifest
- [x] Migration safety (additive, requires .lexicon/ exists first)

### 20e: Mode-Aware Commands
- [x] `lexicon workspace verify` — workspace-level verification
- [x] `lexicon workspace doctor` — workspace-level health checks
- [x] `lexicon ecosystem verify` — ecosystem-level verification
- [x] `lexicon ecosystem doctor` — ecosystem-level health checks
- [x] `lexicon verify` adapts output based on current mode (shows workspace/ecosystem hints)
- [x] `lexicon doctor` adapts output based on current mode (shows workspace/ecosystem hints)

### 20f: Architecture Governance
- [x] Crate roles (Foundation, Interface, Adapter, Application, Utility, Test) in spec types
- [x] Dependency rules (from_role → allowed/forbidden targets) in spec types
- [x] Role inference from crate names in workspace init
- [x] Default dependency rules created during workspace init
- [x] Architecture rules written to `.lexicon/architecture/rules.toml`

### 20g: Mode-Aware TUI
- [x] Dashboard shows current mode badge (Repo/Workspace/Ecosystem) in title
- [x] Architecture tab shows crate roles, dependency rules, architecture graph from workspace manifest
- [x] Tabs appear/disappear based on active mode (Architecture tab only in Workspace/Ecosystem)
- [x] Mode detection via `detect_mode()` in TUI app state

### 20h: Mode-Aware Conversation Loops
- [x] `mode_hints.rs` — mode-specific conversation hints
- [x] Repo Mode: public API surface, test tag questions
- [x] Workspace Mode: additionally crate scope, shared interface, dependency direction questions
- [x] Ecosystem Mode: additionally cross-repo, repo ownership questions
- [x] `init_mode_description()` for mode-appropriate init flow descriptions

### 20i: Tests & Documentation
- [x] Tests for repo shape detection (4 shape tests, 4 mode tests)
- [x] Tests for mode selection (OperatingMode serde, display, default)
- [x] Tests for workspace init, verify, doctor (6 tests)
- [x] Tests for ecosystem init, verify, doctor (6 tests)
- [x] Tests for mode-aware conversation hints (4 tests)
- [x] Tests for capability sets (4 tests)
- [x] Docs: "Progressive Adoption" concept page with ProgressiveScopeDiagram
- [x] Docs: workspace and ecosystem command reference pages
- [x] Sidebar updated with Governance section and Progressive Adoption concept

---

## Phase 21: Documentation Diagrams

Implements `specs/prompts/007-docs-diagrams.md`.

### 21a: Diagram Components
- [x] `docs/src/components/diagrams/LexiconModelDiagram.astro` — core concept relationships
- [x] `docs/src/components/diagrams/VerificationPipelineDiagram.astro` — verification flow
- [x] `docs/src/components/diagrams/ProgressiveScopeDiagram.astro` — Repo → Workspace → Ecosystem
- [x] `docs/src/components/diagrams/AISafetyDiagram.astro` — AI boundary model
- [x] `docs/src/components/diagrams/ArchitectureGovernanceDiagram.astro` — architecture rules
- [x] `docs/src/components/diagrams/ContractCoverageDiagram.astro` — coverage mapping

### 21b: Diagram Design
- [x] Inline SVG, theme-aware (light/dark)
- [x] Responsive layout, mobile legible
- [x] Consistent visual style (soft gradients, rounded containers, clean typography)

### 21c: Docs Integration
- [x] Lexicon model diagram on homepage and core concepts page
- [x] Verification pipeline in verification docs and homepage
- [x] Progressive scope diagram in progressive adoption concept page
- [x] AI safety diagram in AI agents docs
- [x] Architecture governance diagram in architecture docs
- [x] Contract coverage diagram in coverage docs

---

## Phase 22: Polished Homepage

Implements `specs/prompts/008-homepage.md`.

### 22a: Homepage Structure
- [x] Custom hero section (title, value proposition, CTAs)
- [x] Visual concept section with embedded LexiconModelDiagram
- [x] "Why Lexicon Exists" problem statement section
- [x] Core concept highlights (FeatureGrid component)
- [x] Progressive scope section (ScopeCards component)
- [x] AI safety section
- [x] Quick start / calls to action section (CTARow component)

### 22b: Implementation
- [x] Custom `docs/src/content/docs/index.mdx` homepage
- [x] Supporting Astro components (Section, FeatureGrid, ScopeCards, CTARow)
- [x] Starlight splash template integration
- [x] Strong, concise copy

---

## Phase 23: Remaining Polish & Fixes

### 23a: Misc
- [x] `AGENTS.md` already includes requirement to keep docs site up-to-date (rule #11 and dedicated section)
- [x] Update ending state counts in SPRINT.md

---

**Sprint 003 state**: 174→216 tests, 15 crates, 28→43-page docs site, 6 SVG diagram components

---

## Backlog (Not Yet Scheduled)

These are referenced in spec prompts but not yet broken into phases:

- **Ecosystem Governance** (`crates/ecosystem` — new crate): federated contracts, repo roles, cross-repo compatibility, ecosystem-level architecture policies. Described in `006-project-description.md` and `004-progressive-adoption.md`.
- **Workspace Shared Contracts**: shared contract registry within a workspace, contract inheritance/composition.
- **Dependency Law Enforcement**: runtime dependency checking against defined rules, forbidden dependency detection.
- **Impact Analysis**: downstream impact analysis for changes (workspace and ecosystem scope).
- **AI Prompt Refinement**: conversation storage for artifact generation sessions, session replay/refinement, prompt training data collection. Extended from `009-ai-assisted-work.md`.
- **Targeted Improve Commands**: `lexicon contract improve`, `lexicon conformance improve`, `lexicon coverage improve`, `lexicon architecture improve`, `lexicon scoring improve`. From `009-ai-assisted-work.md`.
- **TUI Artifact Review**: AI suggestions as reviewable patches in TUI mode. From `009-ai-assisted-work.md` and `010-ai-prompting.md`.
- **Conversation Storage**: specs call for storing AI conversations to `.lexicon/conversations/` for traceability and refinement. Not yet implemented.

### Phase 19 Gaps (addressed in Sprint 003)

- [x] **`AiProvider` trait semantics**: Renamed to `complete(system, user_message)` for clarity.
- [x] **`load_context` silent failure**: Now returns `(String, Vec<String>)` with warnings surfaced to user.
- [x] **Edit option in patch preview**: Three-way choice (Accept/Edit/Reject) using `dialoguer::Editor`.
- [x] **Multi-artifact generation**: `lexicon generate "..."` generates contract + conformance + behavior from a single intent.
