# lexicon

Contract-driven verification for Rust libraries and workspaces.

## What it does

Lexicon helps you define explicit behavioral contracts for your Rust libraries,
then continuously verify that your code conforms to them. Contracts are structured
specs that describe what a library is supposed to do. Conformance suites turn
those specs into reusable, executable verification. Scoring functions assign
concrete quality metrics to your codebase based on contract coverage, test
results, and structural health.

Gates enforce no-regression policies by failing when scores drop below
configured thresholds. You wire gates into CI or run them locally before
merging. The result is a concrete, auditable quality baseline that only moves
forward.

Every scaffolding command -- `init`, `contract new`, `score init`, `gate init`
-- opens an interactive, AI-guided conversation loop. The tool asks structured
questions, proposes an initial artifact, and lets you refine it conversationally.
Context from past contracts, scoring preferences, and naming conventions is
preserved and reused so the process improves over time.

## Quick start

```sh
# Initialize lexicon in your repo
lexicon init

# Create a new contract
lexicon contract new

# Set up the scoring model
lexicon score init

# Set up verification gates
lexicon gate init

# Run verification (scoring + gates)
lexicon verify

# Sync the generated CLAUDE.md context file
lexicon sync claude

# Launch the terminal UI
lexicon tui
```

## Workspace structure

| Crate | Purpose |
|---|---|
| `lexicon-spec` | Contract and spec data models |
| `lexicon-fs` | File system operations and path management |
| `lexicon-repo` | Repository detection and workspace analysis |
| `lexicon-audit` | Audit trail and change tracking |
| `lexicon-scoring` | Scoring model definitions and evaluation |
| `lexicon-gates` | Gate policies and enforcement |
| `lexicon-conformance` | Conformance suite generation and sync |
| `lexicon-conversation` | Conversational artifact refinement loops |
| `lexicon-ai` | AI provider integration |
| `lexicon-scaffold` | Template rendering and file scaffolding |
| `lexicon-core` | Shared types and configuration |
| `lexicon-tui` | Terminal UI (ratatui) |
| `lexicon-cli` | CLI entry point and command dispatch (clap) |

## Development

Development tasks are managed with [just](https://github.com/casey/just).

```sh
just check       # fmt-check + lint + test
just test        # cargo test --workspace
just lint        # cargo clippy --workspace -- -D warnings
just fmt         # cargo fmt --all
just build       # cargo build --workspace
just run <args>  # cargo run -- <args>
just tui         # launch the TUI
just watch       # watch for changes and re-run tests
just loc         # count lines of code (tokei)
```

Minimum supported Rust version: **1.85**.

## License

MIT OR Apache-2.0
