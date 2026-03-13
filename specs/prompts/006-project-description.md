# LEXICON — MASTER IMPLEMENTATION PROMPT

You are implementing a new Rust developer tool called **Lexicon**.

Lexicon is a CLI/TUI system for defining, verifying, and governing the **behavioral law of software systems**.

The tool must support:

- contract-driven development
- reusable conformance suites
- scoring and verification gates
- API drift detection
- contract coverage analysis
- architecture governance
- workspace and ecosystem validation
- AI-assisted development with safety boundaries
- progressive scaling from single repo to large ecosystem

Lexicon must be production-grade Rust software.

It must not be a toy project.

---

# PRODUCT VISION

Lexicon defines the **language of law** for a repository.

This language describes:

- what the system is supposed to do
- how behavior is verified
- how regressions are prevented
- how architecture is enforced
- how AI can safely participate in development

Lexicon combines ideas from:

- specification-driven development
- contract testing
- architecture governance
- AI-assisted workflows
- verification pipelines

The system must be usable in:

1. a small single-crate repo
2. a multi-crate workspace
3. a multi-repo ecosystem

---

# CORE CONCEPTS

Lexicon is composed of several core layers.

These layers form the behavioral law of a codebase.

## Contracts

Contracts define intended behavior.

Contracts describe:

- capabilities
- invariants
- required semantics
- forbidden behavior
- edge cases
- examples
- non-goals
- stability level

Contracts live in:

specs/contracts/

Contracts contain clauses representing behavioral rules.

Example clause:

clauses:
  - id: value_retrievable
    description: values stored must be retrievable
    test_tags:
      - conformance.value_retrieval

---

## Conformance

Conformance proves implementations satisfy contracts.

Conformance is implemented as reusable harnesses.

Examples:

- trait conformance suites
- protocol conformance tests
- backend adapter validation

Location:

tests/conformance/

Conformance must support multiple implementations of the same contract.

---

## Behavior

Behavior scenarios describe system behavior narratively.

Examples:

- acceptance criteria
- regression scenarios
- user-visible expectations

Behavior does NOT replace contracts.

Contracts define the law.
Behavior illustrates the law.

Location:

specs/behavior/

---

## API Surface

Lexicon must inspect the actual public API of the codebase.

Detect:

- public structs
- public enums
- public traits
- public functions
- modules
- methods

Command:

lexicon api scan

Store results in:

.lexicon/api/current.json

Detect API drift with:

lexicon api diff

This compares:

current API
vs
baseline API

Lexicon must detect:

- added public items
- removed public items
- signature changes
- visibility changes

---

## Contract Coverage

Contract coverage measures how much of a contract is tested.

Each contract clause can reference test tags.

Example:

clauses:
  - id: deletion_idempotent
    test_tags:
      - conformance.delete_behavior

Tests can declare tags.

Lexicon computes coverage metrics:

- clauses covered
- clauses missing tests
- coverage %

Command:

lexicon coverage report

Coverage must affect scoring.

---

## Gates

Gates are hard requirements.

Examples:

- formatting
- linting
- unit tests
- conformance tests
- behavior tests
- API compatibility checks

Gate definitions live in:

specs/gates/

Gates are pass/fail.

They prevent regressions.

---

## Scoring

Scoring evaluates overall system quality.

Unlike gates, scoring is weighted.

Example categories:

- correctness
- contract coverage
- test health
- documentation completeness
- lint cleanliness
- API stability

Score definitions live in:

specs/scoring/

Example score output:

correctness: 40/40  
contract_coverage: 12/15  
gates: 20/20  
docs: 4/5  

Final score: 76/80

Command:

lexicon score explain

---

## Architecture

Architecture describes system structure.

Rules include:

- crate roles
- dependency directions
- layering
- visibility rules

Location:

.lexicon/architecture/

Lexicon must detect:

- forbidden dependencies
- layer violations
- architecture drift

Command:

lexicon architecture graph

---

## Ecosystem Governance

Large platforms span multiple repos.

Lexicon must support:

- shared contracts
- repo roles
- cross-repo compatibility checks
- dependency law
- architecture governance

Location:

.lexicon/ecosystem/

Commands:

lexicon ecosystem verify  
lexicon ecosystem doctor

---

# PROGRESSIVE OPERATING MODES

Lexicon must scale progressively.

## Repo Mode

Default mode.

Supports:

- contracts
- conformance
- scoring
- gates
- API scanning
- contract coverage

Used for small repos.

---

## Workspace Mode

For Cargo workspaces.

Adds:

- crate roles
- dependency law
- workspace architecture graph
- shared contracts

Commands:

lexicon workspace init  
lexicon workspace verify  
lexicon workspace doctor

---

## Ecosystem Mode

For multi-repo systems.

Adds:

- repo roles
- federated contracts
- cross-repo verification
- ecosystem architecture rules

Commands:

lexicon ecosystem init  
lexicon ecosystem verify  
lexicon ecosystem doctor

---

# CLI COMMANDS

Core commands:

lexicon init  
lexicon verify  
lexicon doctor  
lexicon tui

Contracts:

lexicon contract new  
lexicon contract edit  
lexicon contract lint  
lexicon contract list

Conformance:

lexicon conformance add  
lexicon conformance sync

Behavior:

lexicon behavior add  
lexicon behavior sync

API:

lexicon api scan  
lexicon api diff

Coverage:

lexicon coverage report

Scoring:

lexicon score init  
lexicon score explain

Gates:

lexicon gate init

Workspace:

lexicon workspace init  
lexicon workspace verify

Ecosystem:

lexicon ecosystem init  
lexicon ecosystem verify

AI workflows:

lexicon improve  
lexicon sync claude

Auth:

lexicon auth login  
lexicon auth status  
lexicon auth refresh  
lexicon auth logout

---

# AUTHENTICATION (PORT FROM HOLOARCH)

Lexicon must reuse the authentication implementation from **holoarch**.

Port the code from:

src/auth.rs  
src/commands/auth.rs

Behavior:

lexicon auth login

Flow:

1. user runs login command
2. CLI opens browser
3. OAuth provider login occurs
4. local HTTP server receives callback
5. CLI exchanges code for token
6. token stored locally

Providers:

- Claude (Anthropic)
- OpenAI

Token storage:

.lexicon/auth/

Example:

.lexicon/auth/claude.json

Commands:

lexicon auth login  
lexicon auth status  
lexicon auth refresh  
lexicon auth logout

AI features must check authentication before execution.

---

# AI CONTEXT

Lexicon generates AI context files.

Example:

CLAUDE.md

This context explains:

- contracts
- architecture rules
- scoring expectations
- gates
- safe modification rules

Command:

lexicon sync claude

---

# AI SAFETY RULES

AI tools must never:

- weaken contracts
- delete tests to satisfy gates
- bypass architecture rules
- silently change score weights
- ignore dependency law

AI-generated changes must always respect the Lexicon.

---

# FILE STRUCTURE

Example repo structure:

.lexicon/
  manifest.toml
  context/
  conversations/
  api/
  auth/
  architecture/
  ecosystem/

specs/
  contracts/
  behavior/
  scoring/
  gates/

tests/
  conformance/
  behavior/
  integration/

fuzz/
benches/

CLAUDE.md

---

# RUST ARCHITECTURE

Implement Lexicon as a Rust workspace.

Recommended crates:

crates/cli  
crates/tui  
crates/core  
crates/spec  
crates/scaffold  
crates/conversation  
crates/conformance  
crates/gates  
crates/scoring  
crates/ai  
crates/fs  
crates/repo  
crates/audit  
crates/api  
crates/coverage  
crates/architecture  
crates/ecosystem

---

# IMPLEMENTATION REQUIREMENTS

You must:

1. design schema structures
2. build CLI command system
3. implement TUI interface
4. implement contract system
5. implement conformance generation
6. implement scoring engine
7. implement gate verification
8. implement API scanner and diff
9. implement contract coverage
10. implement architecture graph
11. implement workspace and ecosystem verification
12. port holoarch auth system
13. implement AI context generation
14. add conversation-driven artifact creation
15. include tests
16. include documentation

---

# FINAL GOAL

Lexicon should become a system that defines and enforces the **law of a software system**.

It must allow both humans and AI agents to evolve code safely while preserving:

- contracts
- architecture
- test coverage
- ecosystem stability