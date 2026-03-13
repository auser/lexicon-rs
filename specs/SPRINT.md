# Lexicon Sprint Plan

## Phase 0: Workspace Scaffolding
- [x] Convert root Cargo.toml to virtual workspace (resolver = "3")
- [x] Create crates/spec (lib, no internal deps)
- [x] Create crates/fs (lib)
- [x] Create crates/repo (lib)
- [x] Create crates/audit (lib)
- [x] Create crates/scoring (lib)
- [x] Create crates/gates (lib)
- [x] Create crates/conformance (lib)
- [x] Create crates/conversation (lib)
- [x] Create crates/ai (lib)
- [x] Create crates/scaffold (lib)
- [x] Create crates/core (lib)
- [x] Create crates/tui (lib)
- [x] Create crates/cli (binary, move src/main.rs)
- [x] Create xtask/ (binary)
- [x] Verify `cargo build` succeeds

## Phase 1: spec Crate — Domain Types
- [x] version.rs — SchemaVersion type
- [x] common.rs — shared enums (ContractStatus, Stability, Severity, etc.)
- [x] error.rs — SpecError
- [x] manifest.rs — Manifest, ProjectMeta, Preferences, PolicyConfig
- [x] contract.rs — Contract, Invariant, Semantic, EdgeCase, Example, HistoryEntry
- [x] conformance.rs — ConformanceSuite, ConformanceTest, FixtureRef
- [x] behavior.rs — BehaviorScenario
- [x] scoring.rs — ScoreModel, ScoreDimension, ScoreThresholds
- [x] gates.rs — GatesModel, Gate, GateCategory
- [x] session.rs — ConversationSession, SessionStep, Decision
- [x] audit.rs — AuditRecord, AuditAction, Actor
- [x] validation.rs — validate functions per type
- [x] Round-trip serde tests (24 passing)

## Phase 2: fs Crate — Safe File Operations
- [x] safe_write.rs — atomic write with backup
- [x] diff.rs — textual diffs (similar crate)
- [x] patch.rs — managed-block insertion/update
- [x] Tests (16 passing)

## Phase 3: repo + audit Crates
- [x] repo/inspect.rs — scan repo, detect workspace
- [x] repo/layout.rs — RepoLayout struct + discover()
- [x] repo/lexicon_dir.rs — .lexicon/ management
- [x] audit/writer.rs — append audit records
- [x] audit/reader.rs — list/filter/load records
- [x] Tests (9 + 4 = 13 passing)

## Phase 4: scoring + gates + conformance Crates
- [x] scoring/engine.rs — compute_score()
- [x] scoring/explain.rs — human-readable breakdown
- [x] gates/runner.rs — execute gate commands
- [x] gates/result.rs — GateResult type
- [x] gates/policy.rs — enforce required gates
- [x] conformance/generator.rs — generate test code
- [x] conformance/templates.rs — harness templates
- [x] Tests (6 + 8 + 3 = 17 passing)

## Phase 5: conversation Crate — Workflow Engine
- [x] workflow.rs — Workflow trait, step types
- [x] engine.rs — ConversationEngine state machine
- [x] session.rs — session persistence
- [x] driver.rs — TerminalDriver (dialoguer) + MockDriver
- [x] Tests (5 passing)

## Phase 6: ai + scaffold Crates
- [x] ai/context.rs — AI-readable context assembly
- [x] ai/boundary.rs — AiProvider trait + NoOpProvider
- [x] ai/policy.rs — AI edit policy
- [x] scaffold/init.rs — repo initialization
- [x] scaffold/contract.rs — write contract TOML
- [x] scaffold/conformance.rs — write conformance files
- [x] scaffold/scoring.rs — write scoring model
- [x] scaffold/gates.rs — write gates config
- [x] scaffold/claude.rs — CLAUDE.md managed blocks
- [x] Tests (3 + 10 = 13 passing)

## Phase 7: core Crate — Orchestration
- [x] error.rs — CoreError with Diagnostic derive
- [x] init.rs — init_repo() + init_repo_noninteractive()
- [x] contract.rs — contract_new() + contract_new_noninteractive() + contract_list()
- [x] score.rs — score_init(), gate_init(), score_explain()
- [x] verify.rs — full verify pipeline (gates + score + audit)
- [x] sync_claude.rs — CLAUDE.md sync
- [x] Tests (6 passing)

## Phase 8: cli Crate — Commands
- [x] app.rs — clap App definition with all subcommands
- [x] commands/init.rs
- [x] commands/contract.rs (new, list, lint stub)
- [x] commands/score.rs (init, explain)
- [x] commands/gate.rs (init)
- [x] commands/verify.rs — colored gate/score output
- [x] commands/doctor.rs — repo health check
- [x] commands/sync.rs — CLAUDE.md sync
- [x] output.rs — heading, success, warning, error, info, divider

## Phase 9: tui Crate — Terminal UI
- [x] app.rs — AppState, Tab enum, run_tui()
- [x] event.rs — keyboard handling (inline in app.rs)
- [x] ui.rs — draw functions (dashboard, contracts, gates, score, help)
- [x] Tab navigation (Tab/arrow/1-5), refresh (r), quit (q/Esc)

## Phase 10: Tests, Docs, Samples
- [x] Integration tests (2 end-to-end tempdir flows)
- [x] Snapshot tests (insta) — 7 tests across spec, scoring, ai
- [x] Sample contract.toml
- [x] Sample scoring_model.toml
- [x] Sample gates.toml
- [x] Sample conversation_session.json
- [x] Sample audit_record.json
- [x] Sample CLAUDE.md managed block
- [x] README.md
- [x] Architecture doc (reference/architecture.mdx)

## Phase 11: Documentation Site (Astro + Starlight)
- [x] Initialize Astro + Starlight project in docs/ (pnpm)
- [x] Configure sidebar, site title, themes
- [x] Landing page with hero, CardGrid, key features
- [x] Getting Started pages (installation, quickstart)
- [x] Concepts pages (contracts, conformance, scoring, gates, conversations, AI integration)
- [x] Command reference pages (init, contract, score, gate, verify, sync, doctor, tui)
- [x] Schema reference pages (schemas, repo-layout, audit)
- [x] Tutorial guides (first-contract, ci-integration)
- [x] GitHub Actions workflow for Pages deployment
- [x] Justfile recipes (docs-dev, docs-build, docs-preview)
- [x] Pagefind search indexing (23 pages)
- [x] Verify site builds and renders

---

**Total: 103 tests passing across 13 crates, 24-page docs site**
