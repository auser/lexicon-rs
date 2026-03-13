# AGENTS.md

## Purpose

This repository builds `lexicon`, a Rust CLI/TUI for creating and maintaining contract-driven verification systems for Rust libraries and workspaces.

`lexicon` is not a generic scaffolder.

It exists to help users define and evolve:

- stable contracts
- reusable conformance suites
- behavior scenarios
- scoring functions
- hard no-regression gates
- AI-readable repository context
- bounded AI-guided improvement loops

The core philosophy is:

- contract-first
- spec-driven
- reusable conformance over ad hoc tests
- explicit scoring over intuition
- hard gates over soft promises
- AI as a bounded contributor, not an uncontrolled author
- repo-local, inspectable state over hidden magic

---

## Product Principles

All changes in this repository should reinforce these principles:

1. Stable contracts must be explicit.
2. Generated conformance must be reusable.
3. Verification must be explainable.
4. Required gates must be hard to bypass accidentally.
5. AI conversation loops must be useful, structured, and inspectable.
6. Repo-local memory/context must remain visible and reviewable.
7. UX quality matters. The product should feel premium and delightful.
8. TUI support is a first-class feature, not a sidecar.
9. The target repository must not depend on `lexicon` at runtime.
10. The tool must remain usable without hosted infrastructure.

---

## Repository Expectations

This is a production-minded Rust workspace.

Expected high-level crate responsibilities:

- `crates/cli` — CLI entrypoint and command dispatch
- `crates/tui` — terminal UI
- `crates/core` — orchestration/domain services
- `crates/spec` — schemas, parsing, validation, migration
- `crates/scaffold` — generation/scaffolding logic
- `crates/conversation` — conversational refinement loops/session state
- `crates/conformance` — conformance generation/domain logic
- `crates/gates` — verify runners/result normalization
- `crates/scoring` — score model and evaluation
- `crates/ai` — AI prompt/context generation and integration boundaries
- `crates/fs` — safe file writes, patching, diffs
- `crates/repo` — repository inspection and analysis
- `crates/audit` — audit records and history
- `xtask/` — dev automation

Do not collapse this into a monolith unless there is a very strong reason.

---

## How Agents Should Work In This Repo

When making changes:

1. Understand the product intent first.
2. Preserve crate boundaries.
3. Prefer incremental, reviewable changes.
4. Keep code compiling after each phase.
5. Add or update tests with meaningful coverage.
6. Update docs when behavior or architecture changes.
7. Preserve deterministic core behavior.
8. Keep AI features bounded and inspectable.
9. Keep generated files explainable.
10. Do not introduce hidden state or magical behavior.

---

## Non-Negotiable Rules

### 1. Do not weaken safety boundaries casually
Do not loosen policy, gate enforcement, or audit guarantees just to make a flow easier.

### 2. Do not hide complexity in placeholders
Avoid fake implementations that pretend features exist when they do not.

### 3. Do not blur stable contract and speculative guidance
Stable, required behavior must remain clearly distinct from notes, suggestions, or implementation hints.

### 4. Do not make AI mandatory for core functionality
Core repository verification and artifact loading should still work locally without live AI assistance.

### 5. Do not create opaque memory
Repo-local context, summaries, and conversation state must remain inspectable in files.

### 6. Do not silently mutate user-authored meaning
If a command materially changes contract semantics, score thresholds, or gate definitions, the UX must make that clear.

### 7. Do not bypass tests or policy to “get green”
Never delete or weaken tests, scoring, or gates to make the suite pass unless that change is explicitly intentional and visible.

### 8. Do not over-centralize logic in the CLI crate
Business logic belongs in domain crates.

### 9. Do not treat the TUI as optional polish
Interactive UX is a core product requirement.

### 10. Do not introduce a hosted dependency model
Assume the user wants local, inspectable, repo-centric workflows.

### 11. Do not leave the documentation site out of date
When changing commands, schemas, workflows, or user-facing behavior, the corresponding Astro+Starlight documentation pages must be updated in the same changeset. Documentation drift is a bug.

---

## AI Conversation Loop Requirements

The conversation loop is a core feature.

For commands such as:

- repo init
- contract creation
- conformance generation
- score initialization
- gate initialization

the system should prefer a structured conversational workflow over one-shot template emission.

A valid conversation loop should generally:

1. inspect local repo context
2. gather structured inputs
3. propose an artifact draft
4. present the draft clearly
5. allow iterative refinement
6. preserve a summary of decisions
7. validate the result
8. write the final artifact safely

Conversation memory must be:

- repo-local
- inspectable
- summarized
- scoped to the workflow
- reusable in future artifact generation

Do not implement a vague general chat system when a workflow-oriented conversation model is more appropriate.

---

## Verification and Safety Rules

This repository must strongly protect against bad or misleading changes.

Be especially careful around:

- score threshold changes
- gate definition changes
- contract stability/status changes
- behavior/conformance mismatches
- skipped or disabled tests
- silent weakening of assertions
- baseline benchmark changes
- AI prompt/context files that alter policy

Where possible, make sensitive changes explicit in code and UX.

---

## File and Schema Discipline

When changing schemas or generated file formats:

1. version the schema
2. document the change
3. add migration handling where appropriate
4. update examples
5. update tests
6. update docs that reference the format

Do not casually break previously generated repo state.

---

## Preferred Development Style

Prefer:

- explicit data models
- typed domain boundaries
- deterministic transformations
- layered architecture
- clean error handling
- strong validation
- golden/sample artifacts where useful
- focused modules
- documented decisions
- testable interfaces

Avoid:

- hidden mutable globals
- CLI-only business logic
- sprawling untyped config handling
- brittle stringly-typed orchestration
- magic conventions without documentation
- giant god-modules
- “temporary” hacks that become architecture

---

## Testing Expectations

Changes should include appropriate test coverage.

Relevant test categories include:

- schema parsing/validation
- migration tests
- scaffolding tests
- diff/safe-write tests
- conversation state tests
- score calculation tests
- gate normalization tests
- repo inspection tests
- TUI state/model tests where practical
- end-to-end command happy paths

When adding generation behavior, include snapshot/golden-style tests where useful.

---

## Documentation Expectations

Update documentation when changing:

- command structure
- schema shape
- generated repo layout
- conversation model
- scoring semantics
- gate semantics
- Claude context behavior
- audit record behavior

Important docs to maintain:

- `README.md`
- architecture docs
- examples
- sample generated repo output
- managed `CLAUDE.md` examples

---

## Astro + Starlight Documentation Site

This project maintains an Astro + Starlight documentation site. Keeping it current is a first-class requirement, not an afterthought.

When making changes that affect any of the following, the corresponding documentation pages must be updated as part of the same work:

- command structure or CLI behavior
- schema shape or generated file formats
- generated repo layout
- conversation model or workflow steps
- scoring semantics or gate semantics
- Claude context behavior
- audit record behavior
- architecture or crate responsibilities

Treat documentation site updates as part of the definition of done. A feature is not complete until the docs reflect it.

---

## Decision Heuristics

When there are multiple valid choices, prefer the one that is:

1. more explicit
2. more inspectable
3. more reusable
4. safer for AI-assisted workflows
5. easier to test
6. more coherent with the rest of the architecture
7. more pleasant for end users

---

## Sprint Tracking

Agents MUST maintain `specs/SPRINT.md` and keep it up-to-date throughout all building activity.

This file should reflect:

- the current sprint goal or focus area
- what has been completed so far
- what is in progress
- what remains to be done
- any blockers or open questions

Update `specs/SPRINT.md` as work progresses — not just at the start or end. Every meaningful phase of building (feature started, module wired up, tests passing, refactor complete, etc.) should be reflected in the sprint file so that any agent or human picking up work has an accurate picture of current state.

Do not let `specs/SPRINT.md` go stale. If the sprint goal changes, update it. If work is blocked, note it. If a task is done, mark it done.

---

## Change Policy for Agents

When making meaningful changes:

- explain the rationale briefly
- keep patches reviewable
- do not expand scope unnecessarily
- preserve compileability where possible
- avoid unrelated refactors unless clearly justified

If behavior is ambiguous:

- prefer preserving existing contract
- prefer preserving stricter safety
- prefer adding explicit configuration rather than hidden inference

---

## Claude Context Management

This repo will likely generate `CLAUDE.md` managed blocks for target repositories.

When working on those features:

- keep managed regions explicit
- avoid overwriting unmanaged user content
- make updates predictable
- preserve idempotency
- clearly separate generated context from user-authored context

---

## Long-Term Product Direction

The product should grow toward:

- better contract authoring
- better conformance reuse
- better drift detection
- better repo-local memory/context
- better AI workflow discipline
- better colorful interactive UX
- better trustworthiness and auditability

Do not steer the product toward a generic AI coding shell.
It should remain a focused tool for contract-driven engineering systems.

---

## Summary

Build `lexicon` as a serious, opinionated, colorful, interactive Rust tool for repeatable contract-driven verification with structured AI-assisted conversations.

Every meaningful contribution should make the tool:

- safer
- clearer
- more reusable
- more inspectable
- more delightful
- more production-ready