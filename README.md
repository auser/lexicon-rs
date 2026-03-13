# lexicon

**Contract-driven verification for Rust libraries and workspaces.**

Lexicon helps you define explicit behavioral contracts for your Rust code, generate conformance test suites from those contracts, enforce no-regression gates, and compute auditable quality scores — all from the command line.

Every contract is a structured TOML spec that says what your code *must do*, *must not do*, and *how it should behave at the edges*. Conformance suites turn those specs into reusable, executable tests. Gates enforce that quality only moves forward. Scoring gives you a single, explainable number for codebase health.

When AI agents work on your code, Lexicon keeps them aligned: it syncs contract context into `CLAUDE.md`, enforces edit policies, detects gate weakening, and maintains an audit trail of every change.

## Install

```sh
# From crates.io
cargo install lexicon-rs

# Or via the install script
curl -fsSL https://raw.githubusercontent.com/auser/lexicon-rs/main/install.sh | bash
```

**Minimum supported Rust version: 1.85** (edition 2024)

## Quick start

```sh
# Initialize lexicon in your repo
lexicon init

# Create a new behavioral contract
lexicon contract new

# Generate conformance tests from contracts
lexicon conformance add

# Set up the scoring model
lexicon score init

# Set up verification gates
lexicon gate init

# Run full verification (gates + scoring)
lexicon verify

# Launch the terminal UI dashboard
lexicon tui
```

## How it works

Lexicon is built around a layered verification model. Each layer builds on the previous one:

```
┌─────────────────────────────────────────────┐
│  AI Context       CLAUDE.md sync, edit      │
│                   policy, audit trail        │
├─────────────────────────────────────────────┤
│  Ecosystem        Multi-repo governance,    │
│                   shared contracts           │
├─────────────────────────────────────────────┤
│  Architecture     Crate roles, layering     │
│                   rules, dependency law      │
├─────────────────────────────────────────────┤
│  Scoring          Weighted dimensions,      │
│                   pass/warn/fail verdicts    │
├─────────────────────────────────────────────┤
│  Gates            No-regression checks,     │
│                   required/scored/advisory   │
├─────────────────────────────────────────────┤
│  Coverage         Clause-level measurement  │
│                   of contract test coverage  │
├─────────────────────────────────────────────┤
│  Conformance      Generated test suites     │
│                   from contract specs        │
├─────────────────────────────────────────────┤
│  Contracts        Behavioral specs in TOML  │
│                   (invariants, semantics)    │
└─────────────────────────────────────────────┘
```

## Contracts

A contract is a TOML file that describes the behavioral guarantees of a component. Contracts live at `specs/contracts/<id>.toml` and contain:

- **Invariants** — conditions that must always hold (required or advisory severity)
- **Required semantics** — behavior the implementation must provide
- **Forbidden semantics** — behavior that is explicitly prohibited
- **Edge cases** — boundary scenarios with expected behavior
- **Examples** — code snippets demonstrating correct usage
- **Status** lifecycle: `draft` → `active` → `deprecated` → `retired`
- **Stability** levels: `experimental` → `unstable` → `stable` → `frozen`

Example contract (abbreviated):

```toml
[schema_version]
major = 1
minor = 0

id = "key-value-store"
title = "Key-Value Store Contract"
status = "active"
stability = "stable"
scope = "Behavioral guarantees of a concurrent in-memory key-value store."

capabilities = [
    "get/set/delete individual keys",
    "atomic compare-and-swap",
    "TTL-based key expiration",
]

non_goals = [
    "Persistence to disk",
    "Distributed replication",
]

[[invariants]]
id = "inv-001"
description = "A key set with a value must return that exact value on subsequent get."
severity = "required"

[[required_semantics]]
id = "req-001"
description = "get(key) returns None for keys that have never been set."
test_tags = ["conformance", "basic"]

[[forbidden_semantics]]
id = "forbid-001"
description = "Must not panic on get or delete of a missing key."
test_tags = ["safety"]

[[edge_cases]]
id = "edge-001"
scenario = "Setting a key with an empty string value"
expected_behavior = "The key is stored; get returns Some(\"\")."
```

See [`samples/contract.toml`](samples/contract.toml) for a full example.

## Conformance suites

Conformance suites are generated test harnesses that verify your code against a contract. Lexicon supports two styles:

**Trait-based** (default) — your implementation provides a trait with a `create_instance()` method:

```rust
// tests/conformance/key_value_store.rs (generated)
pub trait KeyValueStoreConformance {
    fn create_instance() -> Self;
}

#[test]
fn inv_001_get_returns_set_value() {
    let store = MyStore::create_instance();
    store.set("k", "v");
    assert_eq!(store.get("k"), Some("v".to_string()));
}
```

**Factory-based** — uses a standalone factory function for simpler cases.

Generate and sync suites with:

```sh
lexicon conformance add    # Generate new suite from a contract
lexicon conformance sync   # Re-sync suites when contracts change
```

## Gates

Gates are no-regression checks defined in `specs/gates.toml`. Each gate runs a shell command and reports pass/fail:

```toml
[[gates]]
id = "fmt"
command = "cargo fmt --all -- --check"
category = "required"      # required | scored | advisory
timeout_secs = 60

[[gates]]
id = "clippy"
command = "cargo clippy --workspace -- -D warnings"
category = "required"

[[gates]]
id = "tests"
command = "cargo test --workspace"
category = "required"
```

Gate categories control enforcement:
- **required** — must pass; verification fails if any required gate fails
- **scored** — contributes to the quality score
- **advisory** — informational; does not affect the score

Lexicon detects **gate weakening** — if someone downgrades a gate from `required` to `advisory`, it flags the change.

## Scoring

The scoring model assigns a single, explainable quality score to your codebase. Defined in `specs/scoring/model.toml`:

```toml
[[dimensions]]
id = "correctness"
weight = 30
source = "gate"
gate_id = "tests"

[[dimensions]]
id = "conformance-coverage"
weight = 25
source = "coverage"

[[dimensions]]
id = "lint-quality"
weight = 15
source = "gate"
gate_id = "clippy"

[thresholds]
pass = 80
warn = 60
```

Each dimension has a weight and source. The final score produces a verdict: **Pass**, **Warn**, or **Fail**.

```sh
lexicon score init      # Create scoring model interactively
lexicon score explain   # Show how the score is calculated
lexicon verify          # Run gates + compute score
```

## Coverage

Coverage measures how much of each contract's clauses (invariants + required + forbidden semantics) are covered by tests:

```sh
lexicon coverage report         # Human-readable report
lexicon coverage report --json  # Machine-readable output
```

## Public API analysis

Track your public API surface and detect breaking changes:

```sh
lexicon api scan        # Extract current public API
lexicon api baseline    # Save current API as baseline
lexicon api diff        # Compare current API against baseline
lexicon api report      # Full API surface report
```

## AI integration

Lexicon keeps AI agents aligned with your specifications:

- **Context sync** — `lexicon sync claude` generates a `CLAUDE.md` file with contract context, architecture rules, and edit policies
- **Edit policy** — file-level permissions (`allowed`, `requires-review`, `protected`) that constrain what AI can modify
- **Gate weakening detection** — prevents AI from silently downgrading verification thresholds
- **Audit trail** — records before/after state for every AI-driven change
- **Improvement loop** — `lexicon improve --goal <goal>` runs an AI-guided improvement cycle

AI is never required. The core system uses a NoOp AI provider by default and works entirely offline.

## Repository health

```sh
lexicon doctor    # Check repo health, detect drift, surface issues
```

## Terminal UI

Lexicon includes a rich terminal dashboard built with [ratatui](https://github.com/ratatui/ratatui):

```sh
lexicon tui
```

The TUI shows contract status, gate results, scores, and coverage in a single interactive view.

## CLI reference

```
lexicon init                    Initialize lexicon in a repository
lexicon contract new            Create a new contract interactively
lexicon contract list           List all contracts
lexicon contract lint           Validate contract files
lexicon conformance add         Generate conformance suite from contract
lexicon conformance sync        Re-sync suites with updated contracts
lexicon behavior add            Add behavior scenarios
lexicon behavior sync           Sync behavior scenarios
lexicon score init              Create scoring model
lexicon score explain           Explain score calculation
lexicon gate init               Set up verification gates
lexicon verify                  Run all gates and compute score
lexicon improve --goal <goal>   AI-guided improvement loop
lexicon api scan                Extract public API
lexicon api diff                Diff against API baseline
lexicon api baseline            Save current API as baseline
lexicon api report              Full API surface report
lexicon coverage report         Contract coverage report
lexicon doctor                  Check repo health
lexicon sync claude             Sync context to CLAUDE.md
lexicon tui                     Launch terminal UI
```

## Workspace structure

Lexicon is organized into 15 crates across strict dependency layers:

| Layer | Crate | Purpose |
|-------|-------|---------|
| 0 — Types | `lexicon-spec` | Domain types, schemas, and validation |
| 1 — Engines | `lexicon-fs` | Safe atomic file operations and diffs |
| 1 | `lexicon-scoring` | Scoring engine and model evaluation |
| 1 | `lexicon-conformance` | Conformance suite generation |
| 2 — Services | `lexicon-repo` | Repository inspection and workspace analysis |
| 2 | `lexicon-audit` | Audit records and change tracking |
| 2 | `lexicon-gates` | Gate runners and policy enforcement |
| 2 | `lexicon-api` | Public API extraction and diffing |
| 2 | `lexicon-coverage` | Contract coverage analysis |
| 3 — Domain | `lexicon-conversation` | Conversational refinement loops and sessions |
| 3 | `lexicon-scaffold` | Template rendering and file emission |
| 4 — AI | `lexicon-ai` | AI prompt construction and integration |
| 5 — Orchestration | `lexicon-core` | High-level operations and domain services |
| 6 — Frontends | `lexicon-rs` (CLI) | Command-line interface (clap) |
| 6 | `lexicon-tui` | Terminal UI dashboard (ratatui) |

Layers enforce a strict dependency rule: a crate may only depend on crates in lower layers.

## Development

Development tasks are managed with [just](https://github.com/casey/just):

```sh
just check          # fmt-check + lint + test
just test           # cargo test --workspace
just test-crate X   # cargo test -p lexicon-X
just lint           # cargo clippy --workspace -- -D warnings
just fmt            # cargo fmt --all
just build          # cargo build --workspace
just build-release  # cargo build --release
just run <args>     # cargo run -- <args>
just tui            # launch the TUI
just verify         # run verification
just watch          # watch for changes and re-run tests
just loc            # count lines of code (tokei)
just deps           # show workspace dependency tree
just docs-dev       # start docs site in dev mode
just docs-build     # build docs site
just release X.Y.Z  # create release with changelog (git-cliff)
just release-auto   # auto-detect version and release
just clean          # cargo clean
```

## Releasing

Releases use [git-cliff](https://github.com/orhun/git-cliff) for changelog generation:

```sh
# Manual version
just release 0.2.0

# Auto-detect from conventional commits
just release-auto
```

Both commands generate `CHANGELOG.md`, commit it, and create a signed git tag. Push with `git push && git push --tags` to trigger the release workflow, which builds binaries for Linux and macOS (x86_64 + aarch64) and publishes all crates to crates.io.

## License

MIT OR Apache-2.0
